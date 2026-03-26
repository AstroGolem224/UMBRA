# Changelog

All notable changes to UMBRA are documented here.
Format: [semver] - YYYY-MM-DD

## [0.2.0] - 2026-03-26

### Added
- **Agent Workbench**: Full dispatch lifecycle — compose, select agent/workspace/mode/persona, submit to provider CLI (Codex/Claude/Gemini), track runs with events and artifacts, retry/cancel/continue
- **Ops Room**: Native channel-based orchestration — messages, mention routing, jobs, approval queue, route rules, session templates, turn-taking, restart recovery, audit trail, pagination
- **Provider Onboarding**: Inline checklist, auth check, smoke test, workspace bootstrap directly in Workbench and Ops Room
- **Provider Presence**: Agent presence per channel, live run badges on jobs/messages, escalation rail for blocked/needs_input runs
- **Dashboard Overhaul**: KPI bar chart, deployment registry, attention rail, upcoming deadlines, recent activity timeline, telemetry strip
- **First-Run Onboarding Wizard**: Setup wizard for vault path, PM Tool URL, GitHub PAT, repo root on first launch
- **Agent Auth Token Management**: Per-agent token UI in Settings with show/hide/copy/regenerate per agent
- **Windows Build Pipeline**: GitHub Actions workflow (.exe + .msi via tauri-action, triggers on push to main + tags)
- **Notes**: Category filter, autosave, markdown editor with live preview + Mermaid rendering, quick-link insertion for tasks/agents/repos
- **Workspace Manager**: Inline workspace preset editor in Workbench/Ops Room with grant-root validation
- **Threaded Replies**: Parent-threading for ops room messages with reply composer
- **Session Templates**: Multi-agent orchestration workflow templates
- **Browser E2E**: Smoke/E2E coverage for Workbench and Ops Room flows

### Changed
- Dashboard redesigned to Stitch Light Variant 3 design tokens (astraeus_command_light palette)
- Light theme colors aligned to stitch spec (#00658d primary, #21bcff accent, #f7f9fb background)
- Cron card light theme contrast fixed (white text on darker gradient)
- Notes frontmatter parser fixed for tags/created_at/updated_at field mapping
- Performance: content-visibility for heavy offscreen sections (kanban, registry, note-list, skills-grid)
- Version bumped to 0.2.0 across package.json, Cargo.toml, tauri.conf.json

### Fixed
- Notes TypeError when opening notes with tags field parsed as string instead of array
- Skills page loading from ~/.claude/skills/ in browser dev mode
- Tasks PM Tool sync via Vite proxy (975 tasks across 20 projects)
- Unused PM_TOOL_BASE variable in tauri-mock.ts removed

## [0.1.0.0] - 2026-03-17

### Added
- **Phase 2: CronView** — job list with live-log, run-now, enable/disable toggle, add/delete jobs; Rust `tokio-cron-scheduler` backend with `cron-job-ran` Tauri event
- **Phase 2: TasksView** — Kanban board (IN PROGRESS / TODO / BLOCKED / DONE) synced from PM Tool (`http://100.115.61.30:8000`); 30s live-polling via cron scheduler
- **Phase 3: EmberCanvas** — 120-particle ambient background animation, theme-aware, `prefers-reduced-motion` guard
- **Phase 3: Hex Grid** — SVG-pattern CSS overlay on `#umbra-root::after`, theme-aware (ember/neon/light)
- **Phase 3: GitHub API** — `get_github_repos` Rust command with reqwest + 5-min TTL cache + PAT auth; LauncherView shows open issues + last push
- **Phase 4: PluginsView** — Integration cards for Obsidian, TM-lite, GitHub with live status dots and data fetch
- **Phase 4: Obsidian Integration** — `get_obsidian_stats` Rust command: vault FS scan (max 2000 .md), total count + 5 most recently modified notes
- **Phase 4: TM-lite Integration** — `get_tmlite_tasks` Rust command: reads `vault_path/Tasks/*.md`, parses YAML frontmatter, filters out done/cancelled, sorts by priority
- **Phase 4: SkillsView** — Reads real skills from `~/.claude/skills/` via `list_skills` Rust command; shows name, version (from `VERSION` file), description (from `SKILL.md` frontmatter)
- **Phase 4: AgentsView notes** — Notes textarea + Dashboard Link field per agent; persisted in `AppConfig.agentNotes`; opens URL via Tauri `shell:allow-open`
- **Phase 4: Theme Swatches** — Live ember/neon/light swatch picker in Settings replaces static `<select>`; applies instantly via `setTheme()`
- **BRAND.md v2.1 compliance** — Ember palette (`#d4520a`), Barlow Condensed / Inter / JetBrains Mono font stack, radial glow atmosphere, glassmorphism ember-tinted borders
- **Tauri 2 capabilities** — `capabilities/default.json` with `core:window:allow-start-dragging` (fixes frameless window drag), minimize/maximize/close permissions
- **PM Tool URL** — Permanently set to `http://100.115.61.30:8000` in Rust defaults and frontend config store
- **GitHub PAT field** in SettingsView

### Changed
- `AppConfig` extended with `cron_jobs`, `github_pat`, `agent_notes` (all backward-compatible via serde defaults)
- AppSidebar: TASKS badge shows in-progress count; added PLUGINS + CRON nav items
- DashboardView: loads task store on mount for live badge updates

