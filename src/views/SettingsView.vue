<template>
  <div class="settings-view">
    <ViewHero
      kicker="configuration"
      title="Settings"
      subtitle="theme, vault, launch targets and external service settings."
    >
      <template #meta>
        <span class="view-hero-pill">{{ draft.theme }} active</span>
        <span class="view-hero-pill">{{ draft.closeToTray ? "close hides" : "close exits" }}</span>
        <span class="view-hero-pill">{{ draft.pmToolPollSeconds }}s pm poll</span>
      </template>
    </ViewHero>

    <form class="settings-form" @submit.prevent="save">
      <GlassCard>
        <h3 class="card-title">appearance</h3>
        <div class="field">
          <label class="field-label">theme</label>
          <div class="theme-swatches">
            <button
              v-for="t in themes"
              :key="t.value"
              class="theme-swatch"
              :class="{ active: draft.theme === t.value }"
              type="button"
              @click="applyTheme(t.value)"
            >
              <span class="swatch-dot" :style="{ background: t.color }" />
              {{ t.label }}
            </button>
          </div>
        </div>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">system tray</h3>
        <label class="checkbox-row">
          <input v-model="draft.closeToTray" type="checkbox" />
          <span>hide to tray when the main window is closed</span>
        </label>
        <p class="field-hint">
          tray menu exposes `show`, `hide`, `sync pm now`, and `quit`. when this is off, the window close button exits the app normally.
        </p>
        <label class="checkbox-row" style="margin-top: 12px">
          <input v-model="autostart" type="checkbox" @change="toggleAutostart" />
          <span>start UMBRA automatically when you log in</span>
        </label>
        <p class="field-hint">
          launches minimized in the system tray. uses the OS native autostart mechanism.
        </p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">obsidian vault</h3>
        <div class="field">
          <label class="field-label">vault path</label>
          <input v-model="draft.vaultPath" class="field-input glass-input" type="text" />
        </div>
        <div class="field">
          <label class="field-label">notes subdirectory</label>
          <input v-model="draft.notesSubdir" class="field-input glass-input" type="text" placeholder="UMBRA_Notes" />
        </div>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">launch targets</h3>
        <div v-for="(target, i) in draft.launchTargets" :key="i" class="launch-row">
          <input v-model="target.name" class="glass-input launch-name" placeholder="name" />
          <input v-model="target.path" class="glass-input launch-path" placeholder="executable path" />
          <NeonButton variant="danger" size="sm" ghost @click="draft.launchTargets!.splice(i, 1)">delete</NeonButton>
        </div>
        <NeonButton size="sm" variant="secondary" @click="addLaunchTarget">+ add target</NeonButton>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">workspaces</h3>
        <div v-for="(workspace, index) in draft.workspacePresets" :key="workspace.id" class="workspace-card">
          <div class="workspace-row__head">
            <input v-model="workspace.name" class="glass-input workspace-name" placeholder="workspace name" />
            <label class="checkbox-row compact">
              <input v-model="workspace.writable" type="checkbox" />
              <span>writable</span>
            </label>
            <label class="checkbox-row compact">
              <input
                :checked="draft.defaultWorkspaceId === workspace.id"
                type="radio"
                name="default-workspace"
                @change="draft.defaultWorkspaceId = workspace.id"
              />
              <span>default</span>
            </label>
            <NeonButton variant="danger" size="sm" ghost @click="draft.workspacePresets.splice(index, 1)">delete</NeonButton>
          </div>
          <div class="grant-row">
            <input v-model="workspace.rootPath" class="glass-input" placeholder="absolute workspace path" />
            <NeonButton size="sm" variant="secondary" ghost @click="pickWorkspaceRoot(index)">pick folder</NeonButton>
          </div>
          <div class="workspace-grid">
            <label class="field">
              <label class="field-label">allowed providers</label>
              <input
                :value="workspaceProvidersValue(workspace)"
                class="glass-input"
                placeholder="codex, claude"
                @input="setWorkspaceProviders(workspace, ($event.target as HTMLInputElement).value)"
              />
            </label>
            <label class="field">
              <label class="field-label">allowed agents</label>
              <input
                :value="workspaceAgentsValue(workspace)"
                class="glass-input"
                placeholder="codex-main, claude-main"
                @input="setWorkspaceAgents(workspace, ($event.target as HTMLInputElement).value)"
              />
            </label>
          </div>
        </div>
        <div class="workspace-actions">
          <NeonButton size="sm" variant="secondary" @click="addWorkspacePreset">+ add workspace</NeonButton>
          <NeonButton size="sm" variant="secondary" ghost @click="seedWorkspaceGrantRoots">grant current workspaces</NeonButton>
        </div>
        <div class="field">
          <label class="field-label">workspace grant roots</label>
          <div v-for="(root, index) in draft.workspaceGrantRoots" :key="`${root}-${index}`" class="grant-row">
            <input v-model="draft.workspaceGrantRoots[index]" class="glass-input" placeholder="allowed root path" />
            <NeonButton size="sm" variant="secondary" ghost @click="pickGrantRoot(index)">pick folder</NeonButton>
            <NeonButton variant="danger" size="sm" ghost @click="draft.workspaceGrantRoots.splice(index, 1)">delete</NeonButton>
          </div>
          <NeonButton size="sm" variant="secondary" ghost @click="addWorkspaceGrantRoot">+ add grant root</NeonButton>
        </div>
        <p class="field-hint">
          every run must resolve into one of these roots. UMBRA now rejects dispatches outside this explicit allowlist, even if a workspace preset exists.
        </p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">personas</h3>
        <div v-for="(persona, index) in draft.personaPresets" :key="persona.id" class="persona-row">
          <div class="persona-row__head">
            <input v-model="persona.name" class="glass-input persona-name" placeholder="persona name" />
            <NeonButton variant="danger" size="sm" ghost @click="draft.personaPresets.splice(index, 1)">delete</NeonButton>
          </div>
          <input v-model="persona.description" class="glass-input" placeholder="short description" />
          <textarea
            v-model="persona.prompt"
            class="glass-input persona-prompt"
            rows="4"
            placeholder="system-style prompt fragment for this persona"
          />
        </div>
        <NeonButton size="sm" variant="secondary" @click="addPersonaPreset">+ add persona</NeonButton>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">workbench providers</h3>
        <div class="provider-bootstrap-grid">
          <label class="field">
            <span class="field-label">bootstrap workspace</span>
            <select v-model="providerBootstrapWorkspaceId" class="glass-input">
              <option value="">select workspace</option>
              <option v-for="workspace in workspaceOptions" :key="workspace.id" :value="workspace.id">
                {{ workspace.name }}
              </option>
            </select>
          </label>
          <label class="field">
            <span class="field-label">bootstrap agent</span>
            <select v-model="providerBootstrapAgentId" class="glass-input">
              <option value="">select agent</option>
              <option v-for="agent in bootstrapAgents" :key="agent.id" :value="agent.id">
                {{ agent.name }} · {{ agent.id }}
              </option>
            </select>
          </label>
        </div>
        <label class="checkbox-row compact provider-overwrite-row">
          <input v-model="providerBootstrapOverwrite" type="checkbox" />
          <span>overwrite existing bootstrap files</span>
        </label>
        <div v-for="provider in draft.providerCommands" :key="provider.providerId" class="provider-row">
          <div class="provider-copy">
            <span class="provider-name">{{ providerLabel(provider.providerId) }}</span>
            <span class="provider-hint">{{ provider.providerId }}</span>
            <span class="provider-template">template: {{ providerTemplate(provider.providerId) }}</span>
          </div>
          <input
            v-model="provider.command"
            class="glass-input provider-command"
            :placeholder="provider.providerId"
          />
          <div class="provider-actions">
            <NeonButton size="sm" variant="secondary" ghost @click="checkProviderAuth(provider.providerId)">
              auth
            </NeonButton>
            <NeonButton
              size="sm"
              variant="secondary"
              ghost
              :loading="Boolean(providerActionBusy[provider.providerId])"
              @click="refreshProviderChecklist(provider.providerId)"
            >
              checklist
            </NeonButton>
            <NeonButton
              size="sm"
              variant="secondary"
              ghost
              :loading="Boolean(providerActionBusy[provider.providerId])"
              @click="smokeTestProvider(provider.providerId)"
            >
              smoke
            </NeonButton>
            <NeonButton size="sm" variant="secondary" ghost @click="writeProviderBootstrap(provider.providerId)">
              write bootstrap
            </NeonButton>
            <NeonButton size="sm" variant="secondary" ghost @click="copyProviderEnv(provider.providerId)">
              copy env
            </NeonButton>
            <NeonButton
              size="sm"
              variant="secondary"
              ghost
              :loading="Boolean(probingProviders[provider.providerId])"
              @click="probeProvider(provider.providerId)"
            >
              probe
            </NeonButton>
            <NeonButton size="sm" variant="secondary" ghost @click="openExternal(providerDocs(provider.providerId))">
              docs
            </NeonButton>
          </div>
          <span v-if="providerProbeResults[provider.providerId]" class="provider-result">
            {{ providerProbeResults[provider.providerId]?.summary }}
          </span>
          <span v-if="providerActionResults[provider.providerId]" class="provider-result">
            {{ providerActionResults[provider.providerId] }}
          </span>
          <div v-if="providerChecklists[provider.providerId]" class="provider-checklist">
            <div
              v-for="item in providerChecklists[provider.providerId]?.items ?? []"
              :key="`${provider.providerId}-${item.key}`"
              class="checklist-item"
              :class="{ ready: item.ready, missing: !item.ready }"
            >
              <strong>{{ item.ready ? "ready" : "todo" }}</strong>
              <span>{{ item.label }}</span>
              <span class="provider-result">{{ item.detail }}</span>
            </div>
          </div>
        </div>
        <p class="field-hint">
          set the exact executable UMBRA should launch for each provider. bootstrap actions use the selected workspace + agent, write instruction files into the repo, and can copy a ready worker env with the per-agent UAP token.
        </p>
        <div class="doc-links">
          <NeonButton size="sm" variant="secondary" ghost @click="openExternal(agentSetupGuideUrl)">setup guide</NeonButton>
        </div>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">pm tool</h3>
        <div class="field">
          <label class="field-label">api url</label>
          <input v-model="draft.pmToolUrl" class="field-input glass-input" type="text" placeholder="https://pm-tool.local/api" />
        </div>
        <div class="field">
          <label class="field-label">dashboard url</label>
          <input
            v-model="draft.pmToolDashboardUrl"
            class="field-input glass-input"
            type="text"
            placeholder="https://pm-tool.local"
          />
        </div>
        <div class="field">
          <label class="field-label">poll interval (seconds)</label>
          <input v-model.number="draft.pmToolPollSeconds" class="field-input glass-input" type="number" min="5" max="300" />
        </div>
        <div class="doc-links">
          <NeonButton size="sm" variant="secondary" ghost :disabled="!pmDocsUrl" @click="openExternal(pmDocsUrl)">open api docs</NeonButton>
          <NeonButton size="sm" variant="secondary" ghost :disabled="!pmDashboardUrl" @click="openExternal(pmDashboardUrl)">open tool dashboard</NeonButton>
        </div>
        <p class="field-hint">leave blank if you do not want a live PM integration. docs use the api url, dashboard prefers the explicit dashboard url.</p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">release & updates</h3>
        <div class="field">
          <label class="field-label">updater endpoint</label>
          <textarea
            v-model="draft.updaterEndpoint"
            class="field-input glass-input updater-textarea"
            rows="3"
            placeholder="https://releases.example.com/latest.json"
          />
        </div>
        <div class="field">
          <label class="field-label">updater public key</label>
          <textarea
            v-model="draft.updaterPublicKey"
            class="field-input glass-input updater-textarea"
            rows="4"
            placeholder="CONTENT FROM PUBLICKEY.PEM"
          />
        </div>
        <label class="checkbox-row">
          <input v-model="draft.autoCheckForUpdates" type="checkbox" />
          <span>check for updates on app start</span>
        </label>
        <div class="doc-links">
          <NeonButton size="sm" variant="secondary" ghost :loading="checkingUpdates" @click="checkForUpdates">check now</NeonButton>
          <NeonButton
            size="sm"
            variant="secondary"
            ghost
            :disabled="!canInstallUpdate"
            :loading="installingUpdate"
            @click="installUpdate"
          >
            install pending update
          </NeonButton>
        </div>
        <p class="field-hint">
          {{ updateMessage }}
        </p>
        <p class="field-hint">
          runtime update checks work from these settings. signed updater bundles still require signing env vars during the release build.
        </p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">taskboard</h3>
        <div class="field">
          <label class="field-label">lane defaults</label>
          <div class="lane-pref-list">
            <div v-for="lane in laneOptions" :key="lane.kind" class="lane-pref-row">
              <div class="lane-copy">
                <span class="lane-name">{{ lane.label }}</span>
                <span class="lane-hint">{{ lane.hint }}</span>
              </div>
              <div class="lane-pref-actions">
                <button
                  v-for="mode in laneModes"
                  :key="mode.value"
                  type="button"
                  class="lane-mode"
                  :class="{ active: lanePrefValue(lane.kind) === mode.value }"
                  @click="setLanePref(lane.kind, mode.value)"
                >
                  {{ mode.label }}
                </button>
              </div>
            </div>
          </div>
        </div>
        <p class="field-hint">`smart` keeps UMBRA defaults. `expanded` and `collapsed` override the board startup state explicitly.</p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">uap</h3>
        <div class="field">
          <label class="field-label">advertise host</label>
          <input v-model="draft.uapAdvertiseHost" class="field-input glass-input" type="text" placeholder="127.0.0.1" />
        </div>
        <div class="field-row">
          <div class="field">
            <label class="field-label">port</label>
            <input v-model.number="draft.uapPort" class="field-input glass-input" type="number" min="1" max="65535" />
          </div>
          <div class="field">
            <label class="field-label">legacy shared token</label>
            <input v-model="draft.uapToken" class="field-input glass-input" type="text" autocomplete="off" />
          </div>
        </div>
        <p class="field-hint">runtime auth now uses per-agent tokens. this legacy field is retained for compatibility only.</p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">agent auth tokens</h3>
        <p class="field-hint" style="margin-bottom: 12px">
          each agent authenticates via its own <code>X-Agent-Token</code> header.
          tokens are auto-generated on first run. use <strong>regenerate</strong> to rotate a token — the agent must update its config accordingly.
        </p>
        <div class="token-table">
          <div class="token-row token-row--head">
            <span>agent</span>
            <span>token</span>
            <span>actions</span>
          </div>
          <div v-for="(token, agentId) in agentTokenEntries" :key="agentId" class="token-row">
            <span class="token-agent">{{ agentId }}</span>
            <div class="token-value-cell">
              <input
                :value="tokenVisible[String(agentId)] ? token : '••••••••••••••••'"
                class="glass-input token-input"
                type="text"
                readonly
              />
              <button type="button" class="token-action" title="toggle visibility" @click="toggleTokenVisibility(String(agentId))">
                {{ tokenVisible[String(agentId)] ? "hide" : "show" }}
              </button>
              <button type="button" class="token-action" title="copy to clipboard" @click="copyToken(String(token))">copy</button>
            </div>
            <div class="token-actions-cell">
              <button type="button" class="token-action token-action--danger" @click="regenerateToken(String(agentId))">regenerate</button>
            </div>
          </div>
        </div>
        <p v-if="tokenCopied" class="field-hint" style="color: var(--accent-success)">token copied to clipboard.</p>
      </GlassCard>

      <GlassCard>
        <h3 class="card-title">github</h3>
        <div class="field">
          <label class="field-label">local repo root</label>
          <input
            v-model="draft.repoRootPath"
            class="field-input glass-input"
            type="text"
            placeholder="C:/Repos"
          />
        </div>
        <div class="field">
          <label class="field-label">personal access token</label>
          <input v-model="draft.githubPat" class="field-input glass-input" type="password" placeholder="ghp_..." autocomplete="off" />
        </div>
        <p class="field-hint">token needs <code>public_repo</code> scope, or <code>repo</code> for private repos. launcher all-repos needs a token; local repo actions use the root path above.</p>
      </GlassCard>

      <div class="form-actions">
        <NeonButton type="submit" variant="primary" :loading="saving">save settings</NeonButton>
        <span v-if="saved" class="saved-label">saved.</span>
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import ViewHero from "@/components/layout/ViewHero.vue";
import type {
  AppConfig,
  ProviderAuthState,
  ProviderProbeResult,
  ProviderSetupChecklistResult,
  UpdateCheckResult,
  WorkspaceBootstrapResult,
  WorkspacePreset,
} from "@/interfaces";
import { useAgentStore } from "@/stores/useAgentStore";
import { useConfigStore } from "@/stores/useConfigStore";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";

const agentStore = useAgentStore();
const configStore = useConfigStore();
const saving = ref(false);
const saved = ref(false);
const checkingUpdates = ref(false);
const installingUpdate = ref(false);
const lastUpdateCheck = ref<UpdateCheckResult | null>(null);
const updateError = ref("");
const providerProbeResults = ref<Record<string, ProviderProbeResult | null>>({});
const probingProviders = ref<Record<string, boolean>>({});
const providerActionResults = ref<Record<string, string>>({});
const autostart = ref(false);
const providerActionBusy = ref<Record<string, boolean>>({});
const providerChecklists = ref<Record<string, ProviderSetupChecklistResult | null>>({});
const tokenVisible = ref<Record<string, boolean>>({});
const tokenCopied = ref(false);

const agentTokenEntries = computed(() => {
  const tokens = draft.agentAuthTokens ?? {};
  const sorted = Object.entries(tokens).sort(([a], [b]) => a.localeCompare(b));
  return Object.fromEntries(sorted);
});

function toggleTokenVisibility(agentId: string) {
  tokenVisible.value = { ...tokenVisible.value, [agentId]: !tokenVisible.value[agentId] };
}

async function copyToken(token: string) {
  try {
    await navigator.clipboard.writeText(token);
    tokenCopied.value = true;
    setTimeout(() => { tokenCopied.value = false; }, 2000);
  } catch { /* clipboard not available */ }
}

function regenerateToken(agentId: string) {
  if (!draft.agentAuthTokens) draft.agentAuthTokens = {};
  draft.agentAuthTokens[agentId] = crypto.randomUUID();
}
const providerBootstrapWorkspaceId = ref("");
const providerBootstrapAgentId = ref("");
const providerBootstrapOverwrite = ref(false);

const themes = [
  { value: "ember", label: "ember", color: "#d4520a" },
  { value: "neon", label: "neon", color: "#00f5ff" },
  { value: "light", label: "light", color: "#3b82f6" },
];

const laneOptions = [
  { kind: "backlog", label: "backlog", hint: "smart = collapse only on dense boards" },
  { kind: "in_progress", label: "in progress", hint: "usually best left open" },
  { kind: "review", label: "review", hint: "smart = starts collapsed when populated" },
  { kind: "done", label: "done", hint: "smart = starts collapsed when populated" },
] as const;

const laneModes = [
  { value: "smart", label: "smart" },
  { value: "expanded", label: "expanded" },
  { value: "collapsed", label: "collapsed" },
] as const;

const defaultProviderCommands = [
  { providerId: "codex", command: "codex" },
  { providerId: "claude", command: "claude" },
  { providerId: "gemini", command: "gemini" },
  { providerId: "kimi", command: "kimi" },
];

const defaultPersonaPresets = [
  {
    id: "implementer",
    name: "implementer",
    description: "prefer shipping code, tests, and a concise result summary.",
    prompt: "you are the implementation persona. prefer concrete code changes over discussion. finish with changed files, checks, and blockers.",
  },
  {
    id: "reviewer",
    name: "reviewer",
    description: "prioritize bugs, regressions, missing tests, and trust boundaries.",
    prompt: "you are the review persona. prioritize findings, regressions, missing tests, trust boundaries, and operational risk over summaries.",
  },
  {
    id: "architect",
    name: "architect",
    description: "plan with clear tradeoffs, rollout phases, and failure modes.",
    prompt: "you are the architecture persona. respond with a concrete plan, tradeoffs, rollout order, and critical failure modes before implementation details.",
  },
];

const pmApiUrl = computed(() => draft.pmToolUrl.trim().replace(/\/+$/, ""));
const pmDocsUrl = computed(() => {
  if (!pmApiUrl.value) return "";
  return `${pmApiUrl.value}/docs`;
});
const pmDashboardUrl = computed(() => {
  const explicit = draft.pmToolDashboardUrl.trim().replace(/\/+$/, "");
  if (explicit) return explicit;
  if (!pmApiUrl.value) return "";
  try {
    const url = new URL(pmApiUrl.value);
    if (url.port === "8000") {
      url.port = "4173";
    }
    url.pathname = "";
    url.search = "";
    url.hash = "";
    return url.toString().replace(/\/+$/, "");
  } catch {
    return "";
  }
});
const canInstallUpdate = computed(() => Boolean(lastUpdateCheck.value?.updateAvailable) && !installingUpdate.value);
const updateMessage = computed(() => {
  if (updateError.value) return updateError.value;
  if (!draft.updaterEndpoint.trim() || !draft.updaterPublicKey.trim()) {
    return "set endpoint + public key to enable runtime update checks.";
  }
  if (checkingUpdates.value) return "checking release feed...";
  if (installingUpdate.value) return "installing update. windows will close the app if an update is ready.";
  if (!lastUpdateCheck.value) return "no check yet.";
  if (!lastUpdateCheck.value.configured) return "updater config incomplete.";
  if (lastUpdateCheck.value.updateAvailable) {
    return `update ${lastUpdateCheck.value.version} is ready for install.`;
  }
  return `no update available. current version: ${lastUpdateCheck.value.currentVersion}.`;
});

function applyTheme(t: string) {
  draft.theme = t;
  configStore.setTheme(t);
}

function mergeProviderCommands(commands: AppConfig["providerCommands"] | undefined) {
  const byProvider = new Map(
    (commands ?? [])
      .filter((entry) => entry.providerId?.trim() && entry.command?.trim())
      .map((entry) => [entry.providerId.trim().toLowerCase(), entry.command.trim()]),
  );

  const merged = defaultProviderCommands.map((entry) => ({
    providerId: entry.providerId,
    command: byProvider.get(entry.providerId) ?? entry.command,
  }));

  for (const [providerId, command] of byProvider.entries()) {
    if (!merged.some((entry) => entry.providerId === providerId)) {
      merged.push({ providerId, command });
    }
  }

  return merged;
}

function mergePersonaPresets(personas: AppConfig["personaPresets"] | undefined) {
  const normalized = (personas ?? [])
    .filter((persona) => persona.name?.trim() && persona.prompt?.trim())
    .map((persona) => ({
      id: persona.id?.trim() || crypto.randomUUID(),
      name: persona.name.trim(),
      description: persona.description?.trim() ?? "",
      prompt: persona.prompt.trim(),
    }));

  return normalized.length > 0
    ? normalized
    : defaultPersonaPresets.map((persona) => ({ ...persona }));
}

function providerLabel(providerId: string) {
  switch (providerId) {
    case "codex":
      return "OpenAI Codex";
    case "claude":
      return "Claude Code";
    case "gemini":
      return "Gemini CLI";
    case "kimi":
      return "Kimi";
    default:
      return providerId;
  }
}

function providerDocs(providerId: string) {
  switch (providerId) {
    case "codex":
      return "https://openai.com/index/unlocking-the-codex-harness/";
    case "claude":
      return "https://docs.anthropic.com/en/docs/claude-code/cli-reference";
    case "gemini":
      return "https://github.com/google-gemini/gemini-cli";
    case "kimi":
      return "https://platform.moonshot.ai/docs/guide/agent-support";
    default:
      return "https://github.com";
  }
}

function providerTemplate(providerId: string) {
  switch (providerId) {
    case "codex":
      return "templates/AGENTS.codex.md";
    case "claude":
      return "templates/CLAUDE.md";
    case "gemini":
      return "templates/GEMINI.md";
    default:
      return "templates/worker.env.example";
  }
}

const agentSetupGuideUrl =
  "https://github.com/AstroGolem224/UMBRA/blob/main/docs/agent-setup-guide-2026-03-25.md";
const workspaceOptions = computed(() =>
  (draft.workspacePresets ?? []).filter((workspace) => workspace.name.trim() && workspace.rootPath.trim()),
);
const bootstrapAgents = computed(() =>
  agentStore.agents.filter((agent) => ["online", "idle", "working"].includes(agent.status)),
);

const draft = reactive<AppConfig>({
  ...configStore.config,
  workspaceGrantRoots: [...(configStore.config.workspaceGrantRoots ?? [])],
  personaPresets: mergePersonaPresets(configStore.config.personaPresets),
  providerCommands: mergeProviderCommands(configStore.config.providerCommands),
});

watch(
  () => configStore.config,
  (c) =>
    Object.assign(draft, {
      ...c,
      workspaceGrantRoots: [...(c.workspaceGrantRoots ?? [])],
      personaPresets: mergePersonaPresets(c.personaPresets),
      providerCommands: mergeProviderCommands(c.providerCommands),
    }),
  { deep: true }
);

async function toggleAutostart() {
  try {
    if (autostart.value) {
      await invoke("plugin:autostart|enable");
    } else {
      await invoke("plugin:autostart|disable");
    }
  } catch {
    // Autostart plugin not available in browser mode
  }
}

onMounted(async () => {
  if (!agentStore.agents.length) {
    await agentStore.loadAgents();
  }
  ensureBootstrapSelections();
  await refreshAllProviderChecklists();
  try {
    autostart.value = await invoke<boolean>("plugin:autostart|is_enabled");
  } catch {
    // Not available in browser mode
  }
});

function addLaunchTarget() {
  if (!draft.launchTargets) draft.launchTargets = [];
  draft.launchTargets.push({ id: crypto.randomUUID(), name: "", path: "", icon: "LN" });
}

function addWorkspacePreset() {
  draft.workspacePresets.push({
    id: crypto.randomUUID(),
    name: "",
    rootPath: "",
    writable: true,
    allowedProviders: [],
    allowedAgents: [],
  });
}

function addPersonaPreset() {
  draft.personaPresets.push({
    id: crypto.randomUUID(),
    name: "",
    description: "",
    prompt: "",
  });
}

function addWorkspaceGrantRoot() {
  draft.workspaceGrantRoots = [...(draft.workspaceGrantRoots ?? []), ""];
}

function seedWorkspaceGrantRoots() {
  draft.workspaceGrantRoots = mergeWorkspaceGrantRoots(
    draft.workspaceGrantRoots,
    draft.workspacePresets,
  );
}

function workspaceProvidersValue(workspace: WorkspacePreset) {
  return (workspace.allowedProviders ?? []).join(", ");
}

function setWorkspaceProviders(workspace: WorkspacePreset, value: string) {
  workspace.allowedProviders = parseCsvList(value);
}

function workspaceAgentsValue(workspace: WorkspacePreset) {
  return (workspace.allowedAgents ?? []).join(", ");
}

function setWorkspaceAgents(workspace: WorkspacePreset, value: string) {
  workspace.allowedAgents = parseCsvList(value);
}

function parseCsvList(value: string) {
  return value
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);
}

function mergeWorkspacePresets(presets: AppConfig["workspacePresets"] | undefined) {
  return (presets ?? [])
    .filter((preset) => preset.name?.trim() || preset.rootPath?.trim())
    .map((preset) => ({
      id: preset.id?.trim() || crypto.randomUUID(),
      name: preset.name.trim(),
      rootPath: preset.rootPath.trim(),
      writable: preset.writable !== false,
      allowedProviders: [...new Set((preset.allowedProviders ?? []).map((entry) => entry.trim()).filter(Boolean))],
      allowedAgents: [...new Set((preset.allowedAgents ?? []).map((entry) => entry.trim()).filter(Boolean))],
    }))
    .filter((preset) => preset.name && preset.rootPath);
}

function mergeWorkspaceGrantRoots(
  roots: string[] | undefined,
  presets: AppConfig["workspacePresets"] | undefined,
) {
  const next = new Set<string>();
  for (const root of roots ?? []) {
    if (root.trim()) next.add(root.trim());
  }
  for (const preset of presets ?? []) {
    if (preset.rootPath?.trim()) next.add(preset.rootPath.trim());
  }
  return [...next].sort((left, right) => left.localeCompare(right));
}

function ensureBootstrapSelections() {
  if (!providerBootstrapWorkspaceId.value && workspaceOptions.value.length > 0) {
    providerBootstrapWorkspaceId.value =
      draft.defaultWorkspaceId || workspaceOptions.value[0].id;
  }
  if (!providerBootstrapAgentId.value && bootstrapAgents.value.length > 0) {
    providerBootstrapAgentId.value = bootstrapAgents.value[0].id;
  }
}

async function pickWorkspaceRoot(index: number) {
  const selection = await open({
    directory: true,
    multiple: false,
    title: "select workspace root",
  });
  if (typeof selection === "string") {
    draft.workspacePresets[index].rootPath = selection;
  }
}

async function pickGrantRoot(index: number) {
  const selection = await open({
    directory: true,
    multiple: false,
    title: "select workspace grant root",
  });
  if (typeof selection === "string") {
    draft.workspaceGrantRoots[index] = selection;
  }
}

function lanePrefValue(kind: (typeof laneOptions)[number]["kind"]) {
  const pref = draft.taskLanePrefs?.[kind];
  if (pref === true) return "collapsed";
  if (pref === false) return "expanded";
  return "smart";
}

function setLanePref(kind: (typeof laneOptions)[number]["kind"], value: (typeof laneModes)[number]["value"]) {
  const next = { ...(draft.taskLanePrefs ?? {}) };
  if (value === "smart") {
    delete next[kind];
  } else {
    next[kind] = value === "collapsed";
  }
  draft.taskLanePrefs = next;
}

async function openExternal(url: string) {
  if (!url) return;
  try {
    await shellOpen(url);
  } catch {
    window.open(url, "_blank");
  }
}

async function probeProvider(providerId: string) {
  probingProviders.value = { ...probingProviders.value, [providerId]: true };
  try {
    providerProbeResults.value = {
      ...providerProbeResults.value,
      [providerId]: await invoke<ProviderProbeResult>("probe_provider_command", { providerId }),
    };
  } catch (error) {
    providerProbeResults.value = {
      ...providerProbeResults.value,
      [providerId]: {
        providerId,
        command: "",
        launchable: false,
        summary: `probe failed: ${String(error)}`,
      },
    };
  } finally {
    probingProviders.value = { ...probingProviders.value, [providerId]: false };
  }
}

function requireBootstrapTarget(providerId: string) {
  ensureBootstrapSelections();
  if (!providerBootstrapWorkspaceId.value) {
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: "select a bootstrap workspace first",
    };
    return null;
  }
  return {
    workspaceId: providerBootstrapWorkspaceId.value,
    agentId: providerBootstrapAgentId.value || undefined,
  };
}

async function checkProviderAuth(providerId: string) {
  providerActionBusy.value = { ...providerActionBusy.value, [providerId]: true };
  try {
    const result = await invoke<ProviderAuthState>("get_provider_auth_state", { providerId });
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: result.summary,
    };
  } catch (error) {
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: `auth check failed: ${String(error)}`,
    };
  } finally {
    providerActionBusy.value = { ...providerActionBusy.value, [providerId]: false };
  }
}

async function refreshProviderChecklist(providerId: string) {
  const target = requireBootstrapTarget(providerId);
  if (!target) return;
  providerActionBusy.value = { ...providerActionBusy.value, [providerId]: true };
  try {
    const result = await invoke<ProviderSetupChecklistResult>("check_provider_setup", {
      providerId,
      workspaceId: target.workspaceId,
    });
    providerChecklists.value = {
      ...providerChecklists.value,
      [providerId]: result,
    };
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: result.summary,
    };
  } catch (error) {
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: `checklist failed: ${String(error)}`,
    };
  } finally {
    providerActionBusy.value = { ...providerActionBusy.value, [providerId]: false };
  }
}

async function refreshAllProviderChecklists() {
  if (!providerBootstrapWorkspaceId.value) return;
  for (const provider of draft.providerCommands) {
    await refreshProviderChecklist(provider.providerId);
  }
}

async function smokeTestProvider(providerId: string) {
  const target = requireBootstrapTarget(providerId);
  if (!target) return;
  providerActionBusy.value = { ...providerActionBusy.value, [providerId]: true };
  try {
    const result = await invoke<ProviderProbeResult>("smoke_test_provider_command", {
      providerId,
      workspaceId: target.workspaceId,
    });
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: result.summary,
    };
  } catch (error) {
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: `smoke test failed: ${String(error)}`,
    };
  } finally {
    providerActionBusy.value = { ...providerActionBusy.value, [providerId]: false };
  }
}

async function writeProviderBootstrap(providerId: string) {
  const target = requireBootstrapTarget(providerId);
  if (!target) return;
  providerActionBusy.value = { ...providerActionBusy.value, [providerId]: true };
  try {
    const result = await invoke<WorkspaceBootstrapResult>("bootstrap_provider_workspace", {
      providerId,
      workspaceId: target.workspaceId,
      agentId: target.agentId,
      overwrite: providerBootstrapOverwrite.value,
    });
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: `${result.summary}: ${result.files.join(", ")}`,
    };
    await refreshProviderChecklist(providerId);
  } catch (error) {
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: `bootstrap failed: ${String(error)}`,
    };
  } finally {
    providerActionBusy.value = { ...providerActionBusy.value, [providerId]: false };
  }
}

async function copyProviderEnv(providerId: string) {
  const target = requireBootstrapTarget(providerId);
  if (!target) return;
  try {
    const content = await invoke<string>("get_provider_env_template", {
      providerId,
      agentId: target.agentId,
    });
    await navigator.clipboard.writeText(content);
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: "worker env copied to clipboard",
    };
  } catch (error) {
    providerActionResults.value = {
      ...providerActionResults.value,
      [providerId]: `copy env failed: ${String(error)}`,
    };
  }
}

async function checkForUpdates() {
  checkingUpdates.value = true;
  updateError.value = "";
  try {
    lastUpdateCheck.value = await invoke<UpdateCheckResult>("check_for_updates");
  } catch (error) {
    updateError.value = `update check failed: ${String(error)}`;
  } finally {
    checkingUpdates.value = false;
  }
}

async function installUpdate() {
  if (!canInstallUpdate.value) return;
  installingUpdate.value = true;
  updateError.value = "";
  try {
    const installed = await invoke<boolean>("install_pending_update");
    if (!installed) {
      updateError.value = "no pending update available.";
      return;
    }
    lastUpdateCheck.value = null;
  } catch (error) {
    updateError.value = `update install failed: ${String(error)}`;
  } finally {
    installingUpdate.value = false;
  }
}

async function save() {
  saving.value = true;
  saved.value = false;
  try {
    await configStore.saveConfig({
      ...draft,
      workspacePresets: mergeWorkspacePresets(draft.workspacePresets),
      workspaceGrantRoots: mergeWorkspaceGrantRoots(
        draft.workspaceGrantRoots,
        draft.workspacePresets,
      ),
      personaPresets: mergePersonaPresets(draft.personaPresets),
      providerCommands: mergeProviderCommands(draft.providerCommands),
    });
    saved.value = true;
    setTimeout(() => (saved.value = false), 2000);
  } finally {
    saving.value = false;
  }
}

watch([providerBootstrapWorkspaceId, providerBootstrapAgentId], () => {
  void refreshAllProviderChecklists();
});
</script>

<style scoped>
.settings-view {
  max-width: 860px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.page-kicker,
.card-title,
.field-label {
  margin: 0;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.page-title {
  margin: 0;
  color: var(--text-primary);
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: 800;
  letter-spacing: 0.06em;
}

.page-subtitle,
.field-hint,
.saved-label {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.6;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.card-title {
  margin-bottom: 12px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
}

.field:last-child {
  margin-bottom: 0;
}

.field-row {
  display: grid;
  grid-template-columns: 140px 1fr;
  gap: 12px;
}

.field-input {
  font-size: 13px;
  padding: 10px 12px;
}

.updater-textarea {
  min-height: 88px;
  resize: vertical;
}

.launch-row,
.provider-row {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  flex-wrap: wrap;
  margin-bottom: 8px;
}

.workspace-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 10px;
  padding: 12px;
  border-radius: var(--radius-lg);
  border: 1px solid color-mix(in srgb, var(--glass-border) 80%, transparent);
  background: color-mix(in srgb, var(--bg-secondary) 88%, transparent);
}

.workspace-row__head,
.workspace-actions,
.grant-row,
.provider-bootstrap-grid,
.workspace-grid {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.provider-bootstrap-grid,
.workspace-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  margin-bottom: 10px;
}

.workspace-name {
  flex: 1;
}

.grant-row {
  margin-bottom: 8px;
}

.grant-row .glass-input {
  flex: 1;
}

.persona-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 10px;
  padding: 12px;
  border-radius: var(--radius-lg);
  border: 1px solid color-mix(in srgb, var(--glass-border) 80%, transparent);
  background: color-mix(in srgb, var(--bg-secondary) 88%, transparent);
}

.persona-row__head {
  display: flex;
  gap: 8px;
  align-items: center;
}

.persona-name {
  flex: 1;
}

.persona-prompt {
  min-height: 100px;
  resize: vertical;
}

.provider-copy {
  width: 160px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.provider-name {
  color: var(--text-primary);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.provider-hint {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
}

.provider-template,
.provider-result {
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.5;
}

.launch-name {
  width: 140px;
}

.launch-path {
  flex: 1;
}

.provider-command {
  flex: 1;
}

.provider-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.provider-overwrite-row {
  margin-bottom: 12px;
}

.provider-checklist {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 100%;
  margin-top: 6px;
}

.checklist-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 8px 10px;
  border-radius: var(--radius-lg);
  border: 1px solid color-mix(in srgb, var(--glass-border) 82%, transparent);
  background: color-mix(in srgb, var(--bg-secondary) 92%, transparent);
}

.checklist-item.ready {
  border-color: color-mix(in srgb, var(--accent-success) 32%, transparent);
}

.checklist-item.missing {
  border-color: color-mix(in srgb, var(--accent-warning) 28%, transparent);
}

.form-actions,
.doc-links,
.theme-swatches,
.lane-pref-actions {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.lane-pref-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.lane-pref-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 10px 0;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
}

.lane-pref-row:first-child {
  padding-top: 0;
  border-top: 0;
}

.lane-copy {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.lane-name {
  color: var(--text-primary);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-size: 12px;
}

.lane-hint {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.lane-mode {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 7px 12px;
  border-radius: var(--radius-pill);
  border: none;
  background: color-mix(in srgb, var(--accent) 6%, var(--bg-surface));
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.16s ease, background 0.16s ease;
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.lane-mode:hover,
.lane-mode.active {
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-surface));
  color: var(--accent);
}

.saved-label {
  color: var(--accent-success);
}

.checkbox-row {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: var(--text-secondary);
  font-size: 13px;
}

.checkbox-row.compact {
  gap: 6px;
}

.checkbox-row input {
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
}

.field-hint code {
  font-family: var(--font-mono);
  color: var(--accent-secondary);
  font-size: 11px;
}

.theme-swatch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  border-radius: var(--radius-pill);
  border: none;
  background: color-mix(in srgb, var(--accent) 6%, var(--bg-surface));
  color: var(--text-secondary);
  cursor: pointer;
  transition: color 0.16s ease, background 0.16s ease;
}

.theme-swatch:hover,
.theme-swatch.active {
  background: color-mix(in srgb, var(--accent) 14%, var(--bg-surface));
  color: var(--accent);
}

.swatch-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

:global([data-theme="light"]) .theme-swatch {
  background: rgba(255, 255, 255, 0.88);
  border-color: rgba(15, 23, 42, 0.08);
}

:global([data-theme="light"]) .lane-mode {
  background: rgba(255, 255, 255, 0.9);
  border-color: rgba(15, 23, 42, 0.12);
}

:global([data-theme="light"]) .theme-swatch:hover,
:global([data-theme="light"]) .theme-swatch.active,
:global([data-theme="light"]) .lane-mode:hover,
:global([data-theme="light"]) .lane-mode.active {
  border-color: rgba(8, 145, 178, 0.2);
  background: rgba(240, 249, 255, 0.96);
}

.token-table {
  display: flex;
  flex-direction: column;
}

.token-row {
  display: grid;
  grid-template-columns: 120px minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  padding: 10px 0;
  border-top: 1px solid color-mix(in srgb, var(--glass-border) 62%, transparent);
}

.token-row--head {
  padding-top: 0;
  border-top: 0;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.token-agent {
  font-family: var(--font-display);
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.token-value-cell {
  display: flex;
  gap: 6px;
  align-items: center;
}

.token-input {
  flex: 1;
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 6px 10px;
  letter-spacing: 0.04em;
  user-select: all;
}

.token-actions-cell {
  display: flex;
  gap: 6px;
}

.token-action {
  font-family: var(--font-mono);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  padding: 4px 8px;
  border-radius: var(--radius-pill);
  border: 1px solid color-mix(in srgb, var(--glass-border) 88%, transparent);
  background: color-mix(in srgb, var(--glass-bg) 78%, transparent);
  color: var(--text-secondary);
  cursor: pointer;
  white-space: nowrap;
}

.token-action:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.token-action--danger:hover {
  border-color: var(--accent-error);
  color: var(--accent-error);
}

@media (max-width: 760px) {
  .field-row {
    grid-template-columns: 1fr;
  }

  .launch-row,
  .provider-row,
  .persona-row__head,
  .workspace-row__head {
    flex-direction: column;
    align-items: stretch;
  }

  .provider-bootstrap-grid,
  .workspace-grid {
    grid-template-columns: 1fr;
  }

  .lane-pref-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .launch-name {
    width: 100%;
  }

  .token-row {
    grid-template-columns: 1fr;
  }
}
</style>
