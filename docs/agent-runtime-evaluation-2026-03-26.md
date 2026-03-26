# Native Agent Runtime Manager — Evaluation

## Status: Evaluated, deferred to post-v0.2.0

## Current State

UMBRA already manages agent processes through the **Workbench Runner** (`src-tauri/src/workbench/runner.rs`):

- Spawns provider CLI processes (`codex exec`, `claude -p`, `gemini -p`) as child processes
- Streams stdout/stderr into RunEvent timeline
- Handles process lifecycle (start, monitor, cancel via process kill)
- Persists run state in SQLite (status transitions, artifact extraction)
- Recovers incomplete runs on app restart (`recover_incomplete_runs`)

Additionally, the **UAP heartbeat server** tracks agent presence:

- Agents report status via `POST /api/agents/:id/heartbeat`
- Per-agent auth tokens prevent spoofing
- Task queues allow push-based dispatch
- Stale detection (30min timeout) marks agents offline

## What "Native Runtime Manager" Would Add

1. **Process Ownership** — UMBRA starts and owns the agent process lifecycle (not just dispatch-and-forget)
2. **Health Monitoring** — Periodic health checks, automatic restart on crash
3. **Resource Limits** — CPU/memory caps per agent process
4. **Log Aggregation** — Centralized log collection from all managed agents
5. **Multi-Instance** — Run multiple agent instances concurrently

## Evaluation

### Already Covered
- Process spawning and lifecycle: **done** (runner.rs)
- Status tracking and recovery: **done** (recover_incomplete_runs)
- Per-agent auth and identity: **done** (uap.rs + credentials.rs)
- Task dispatch and result collection: **done** (workbench store)

### Not Yet Needed
- Health monitoring with auto-restart: Current use case is dispatch-based (start, run, finish). Agents don't run as long-lived daemons
- Resource limits: Provider CLIs manage their own resources. UMBRA is an orchestrator, not a container runtime
- Multi-instance: Current architecture supports concurrent runs per agent. No need for instance management

### Recommendation

**Defer.** The current workbench runner + UAP heartbeat architecture covers the actual use cases. A full runtime manager (like systemd/supervisord for agents) adds complexity without clear user value at this stage.

If the need arises (e.g., long-running agent daemons), the recommended approach is:
1. Add a `ManagedAgent` config with restart policy
2. Use `tokio::process::Command` with a supervisor loop
3. Integrate with Windows Service Manager for production deployments

This should be revisited when UMBRA moves from dispatch-based to daemon-based agent management.
