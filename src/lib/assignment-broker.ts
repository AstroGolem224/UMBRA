import type { Agent, Task } from "@/interfaces";

export interface AssignmentSuggestion {
  agent: Agent;
  task: Task;
  score: number;
  reasons: string[];
}

const PRIORITY_SCORE: Record<Task["priority"], number> = {
  urgent: 50,
  critical: 40,
  high: 30,
  medium: 20,
  low: 10,
};

const STATUS_SCORE: Record<Agent["status"], number> = {
  idle: 28,
  online: 22,
  working: 10,
  offline: -100,
  error: -120,
};

function normalize(text: string) {
  return text.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
}

function keywordSet(task: Task) {
  const parts = [
    task.title,
    task.description ?? "",
    task.project,
    ...(task.labels ?? []),
  ];

  return new Set(
    normalize(parts.join(" "))
      .split(/\s+/)
      .filter((token) => token.length >= 3)
  );
}

function overlapScore(taskKeywords: Set<string>, values: string[]) {
  let hits = 0;

  for (const value of values) {
    for (const token of normalize(value).split(/\s+/)) {
      if (token.length >= 3 && taskKeywords.has(token)) hits += 1;
    }
  }

  return hits;
}

function agentLoadPenalty(agent: Agent) {
  return agent.activeTaskId ? 8 : 0;
}

export function buildAssignmentSuggestions(tasks: Task[], agents: Agent[], limit = 4): AssignmentSuggestion[] {
  const candidateTasks = tasks.filter((task) => !["done", "cancelled"].includes(task.status));
  const candidateAgents = agents.filter((agent) => !["offline", "error"].includes(agent.status));
  const suggestions: AssignmentSuggestion[] = [];

  for (const task of candidateTasks) {
    const taskKeywords = keywordSet(task);
    let best: AssignmentSuggestion | null = null;

    for (const agent of candidateAgents) {
      const roleHits = overlapScore(taskKeywords, [agent.role]);
      const skillHits = overlapScore(taskKeywords, agent.skills);
      const toolHits = overlapScore(taskKeywords, agent.allowedTools);
      const directMention = taskKeywords.has(normalize(agent.name));

      const score =
        PRIORITY_SCORE[task.priority] +
        STATUS_SCORE[agent.status] +
        roleHits * 12 +
        skillHits * 9 +
        toolHits * 6 +
        (directMention ? 18 : 0) -
        agentLoadPenalty(agent);

      const reasons = [
        agent.status === "idle" ? "idle now" : `status ${agent.status}`,
        roleHits > 0 ? `role match x${roleHits}` : null,
        skillHits > 0 ? `skill match x${skillHits}` : null,
        toolHits > 0 ? `tool fit x${toolHits}` : null,
        directMention ? "agent named in task" : null,
        agent.activeTaskId ? "already carries active work" : null,
      ].filter(Boolean) as string[];

      if (!best || score > best.score) {
        best = { agent, task, score, reasons };
      }
    }

    if (best) suggestions.push(best);
  }

  return suggestions
    .sort((a, b) => b.score - a.score || PRIORITY_SCORE[b.task.priority] - PRIORITY_SCORE[a.task.priority])
    .slice(0, limit);
}
