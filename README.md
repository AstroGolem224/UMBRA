# UMBRA — Unified Management Board for Runtimes & Agents

Desktop orchestrator for AI agent workflows. Dispatch tasks to Codex, Claude, and Gemini from a single interface, track runs with live event streams, and coordinate multi-agent operations through channels.

Built with Tauri v2 + Vue 3 + Rust.

## Features

- **Agent Workbench** — Dispatch prompts to provider CLIs (Codex, Claude, Gemini), track runs with live stdout/stderr streaming, retry/cancel/continue, artifact inspection
- **Ops Room** — Channel-based multi-agent orchestration with mention routing, jobs, approval queues, session templates, and turn-taking
- **Task Board** — Kanban board synced with PM Tool API, drag-and-drop lanes, inline reorder, priority sort, create/edit/comment
- **Notes** — Obsidian vault integration with autosave, markdown editor, live preview, Mermaid diagrams, quick-link insertion
- **Skills Browser** — Scans `~/.claude/skills/` folders, displays metadata, category/agent filters
- **Dashboard** — KPI charts, deployment registry, attention signals, upcoming deadlines, activity timeline
- **Cron** — Agent-pushed cron job registry via UAP heartbeat, cross-platform execution
- **Global Search** — Ctrl+K spotlight across commands, notes, tasks, agents, skills, repos
- **Settings** — Themes (Ember/Neon/Light), workspace presets, provider commands, persona presets, agent auth tokens, PM Tool integration

## Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) stable toolchain
- At least one provider CLI: `codex`, `claude`, or `gemini`

## Quick Start

```bash
# Install frontend dependencies
npm install

# Browser dev mode (no Rust backend needed)
npm run dev
# Opens http://localhost:1430 with mock APIs

# Full Tauri dev mode (Rust backend + Vue frontend)
cargo tauri dev
```

## Build

```bash
# Production build (.exe + .msi installer on Windows)
cargo tauri build
```

GitHub Actions builds are configured in `.github/workflows/build-windows.yml`.

## Architecture

```
src/              Vue 3 frontend (Pinia stores, views, components)
src-tauri/        Rust backend (Tauri commands, SQLite store, UAP server)
  src/commands/   65 Tauri commands (agents, config, notes, PM, cron, ops room, workbench)
  src/uap.rs      UAP server (heartbeat, task polling, cron sync)
  src/workbench/  Dispatch runner with provider adapters
stitch/           Design tokens (Stitch light theme specification)
docs/             Agent setup guide, architecture specs, roadmaps
templates/        Provider instruction files (AGENTS.md, CLAUDE.md, GEMINI.md)
```

## Configuration

On first launch, the onboarding wizard guides through:

1. **Obsidian vault path** — where notes are stored
2. **PM Tool URL** — project management API endpoint
3. **GitHub PAT** — for repo browsing (optional)
4. **Repo root path** — local git repositories folder

Settings are persisted at `%APPDATA%/umbra/config.json`. Sensitive values (PAT, agent tokens) can be stored in the OS credential manager.

## UAP (UMBRA Agent Protocol)

Agents authenticate via per-agent tokens (`X-Agent-Token` header). Endpoints:

- `POST /api/agents/:id/heartbeat` — register/update agent status
- `GET /api/agents/:id/tasks` — poll for pending tasks
- `POST /api/agents/:id/cron-jobs` — sync cron job state

See `docs/agent-setup-guide-2026-03-25.md` for provider setup instructions.

## License

Private. CMG internal use only.

