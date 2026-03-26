# UMBRA Improvement Priorities

## 1. [BREAKING] Build a Rust Sync Hub for External Data
**Category:** Architecture  
**Priority:** High  
**Effort:** Large (weeks)

**What:** Move PM, GitHub, agent, cron, and notes metadata synchronization into one Rust-side sync hub instead of letting each view trigger its own fetch logic. Persist last-known-good snapshots in app data so the UI always has something stable to render.
**Why:** `DashboardView.vue`, `TasksView.vue`, `CronView.vue`, and the stores currently pull data independently, which creates duplicated loading behavior, stale-state drift, and weak offline behavior. A local-first orchestration app should treat sync as infrastructure, not view code.
**How:** Add a `sync/` domain in `src-tauri` with per-integration fetchers, normalization, timestamps, and persisted snapshots. Expose one `get_sync_snapshot` command plus targeted refresh commands. Let Pinia stores subscribe to one normalized event stream instead of each store inventing its own polling rhythm.

## 2. [BREAKING] Extract PM Board Logic Out of `TasksView.vue`
**Category:** Architecture  
**Priority:** High  
**Effort:** Large (weeks)

**What:** Move lane resolution, move/reorder semantics, project/column loading, and task mutation flows out of `TasksView.vue` into a typed PM domain layer. Keep the view focused on rendering and interaction.
**Why:** `TasksView.vue` currently carries too much domain logic: lane mapping, drag/drop behavior, API orchestration, project lookup, edit flows, and preference persistence all live in one file. That makes the board harder to test, harder to evolve, and brittle against PM API changes.
**How:** Introduce a `usePmBoardStore` plus a Rust-side PM client module that hides quirks like `PATCH /move` versus `PUT /task`. Model lanes, transitions, capability flags, and optimistic updates in one place. Then slim `TasksView.vue` down to a composition layer with presentational subcomponents.

## 3. Ship a First-Run Onboarding Flow in `SettingsView.vue`
**Category:** Feature  
**Priority:** High  
**Effort:** Medium (days)

**What:** Add a real setup wizard for vault path, PM API base URL, PM dashboard URL, GitHub PAT, repo root, and launch targets. Block half-configured use until the minimum working setup is validated.
**Why:** UMBRA is now more machine-neutral, but the first-run experience is still rough because users can land in partially functional views with no guided recovery path. For a personal control panel, setup friction kills trust fast.
**How:** Add a multi-step onboarding modal that runs before normal navigation when required config is missing. Reuse the existing config commands and validation paths, but present them as a guided flow with live checks, example values, and clear pass/fail states.

## 4. Store Secrets in Windows Credential Manager Instead of Plain Config
**Category:** Security  
**Priority:** High  
**Effort:** Medium (days)

**What:** Move sensitive values like GitHub PAT and future integration tokens out of `config.json` and into Windows-native secret storage. Keep only non-sensitive references or capability flags in the app config.
**Why:** The current config hardening improved defaults, but long-lived secrets still do not belong in a plain app config file on disk. UMBRA is explicitly a local orchestration hub, so it should use Windows 11 platform security instead of treating tokens like normal settings.
**How:** Add a Rust secret abstraction backed by Windows Credential Manager or DPAPI and expose `set_secret`, `get_secret_metadata`, and `clear_secret` commands. Update `SettingsView.vue` to show presence/status instead of raw token values, with replace/remove flows.

## 5. Add Minimize-to-Tray and Background Sync Mode
**Category:** Feature  
**Priority:** High  
**Effort:** Medium (days)

**What:** Let UMBRA live in the Windows system tray, keep lightweight background sync alive, and reopen instantly from the tray. This should include basic quick actions like Show, Sync Now, and Quit.
**Why:** UMBRA behaves like a workstation control plane, not a document editor. For that kind of tool, losing visibility when the window closes is wasteful, and re-opening the full app for small checks adds friction.
**How:** Use Tauri’s tray APIs to add tray lifecycle, tray menu actions, and a minimize-to-tray setting. Keep background sync conservative and tied to the sync hub so the app does not invent a second hidden runtime model.

## 6. Add a Global Command Palette and Keyboard Layer
**Category:** UX  
**Priority:** High  
**Effort:** Medium (days)

**What:** Add a command palette for “new note”, “open task board”, “sync PM”, “launch MMC in Godot”, “open latest note”, and similar actions, backed by keyboard shortcuts. Make it the fastest way to drive the app.
**Why:** UMBRA already centralizes actions across notes, tasks, launcher, and settings, but navigation is still mostly view-first. Power users on Windows need a keyboard-first layer, especially in a tool meant to reduce context switching.
**How:** Implement a lightweight command registry in the frontend with stable command IDs and handlers. Add a modal palette, fuzzy matching over commands plus local entities, and wire platform shortcuts like `Ctrl+K`, `Ctrl+N`, and `Alt+1..9`.

## 7. Create a Shared Async UX System for Loading, Error, Empty, and Retry States
**Category:** UX  
**Priority:** High  
**Effort:** Small (hours)

**What:** Standardize loading bars, stale badges, retry affordances, and empty-state copy across `DashboardView.vue`, `TasksView.vue`, `AgentsView.vue`, `NotesView.vue`, and `CronView.vue`. Right now each surface improvises.
**Why:** The app’s visual language is increasingly coherent, but async state handling still feels piecemeal. In a multi-integration control panel, users should instantly understand whether a panel is empty, broken, stale, or still syncing.
**How:** Add a small set of shared UI primitives like `AsyncPanelState`, `InlineErrorState`, and `StaleDataPill`. Make each store expose consistent fields such as `loading`, `lastSync`, `error`, and `isStale`, then swap ad-hoc strings for shared render patterns.

## 8. Add a Notification Center Plus Windows Toast Notifications
**Category:** Feature  
**Priority:** High  
**Effort:** Medium (days)

**What:** Add an in-app notification inbox and optional Windows toast notifications for failed cron jobs, stale agents, updater results, and PM task changes. Notifications should be actionable, not decorative.
**Why:** UMBRA already has useful telemetry, but that signal stays trapped inside the current view. For a personal orchestration tool, important events need to escape the page and reach the user at the right moment.
**How:** Build a small notification store fed by Rust events and sync deltas. Expose toast-worthy events via the Windows notification plugin, and keep a local in-app history so users can inspect what happened after the toast is gone.

## 9. Upgrade `CronView.vue` with Dry-Run, Run History, and Failure Output
**Category:** Feature  
**Priority:** High  
**Effort:** Medium (days)

**What:** Turn `CronView.vue` from passive schedule telemetry into an operational console with “run now”, recent executions, duration, exit status, and captured notes/output. That includes retry actions and per-job failure context.
**Why:** Schedules without execution detail are not trustworthy. If a nightly automation fails, the user should not have to infer what happened from a single status label.
**How:** Extend the Rust cron model to persist recent runs with timestamps, status, runtime, and short output snippets. Then render a per-job drawer or modal in `CronView.vue` with test-run and retry actions using the same event bus as scheduled runs.

## 10. Add Agent Heartbeat Diagnostics and Capability Drift Detection
**Category:** Feature  
**Priority:** High  
**Effort:** Medium (days)

**What:** Expand `AgentsView.vue` to show version, last heartbeat age, current workspace, last reported task, and capability drift such as missing tools or mismatched skill sets. Make agent health more than a simple status badge.
**Why:** An orchestration panel without operational diagnostics is only half useful. When an agent silently stops reporting, loses tools, or diverges from its intended config, the app should explain that clearly.
**How:** Enrich the agent payload with heartbeat metadata and declared-vs-expected capabilities. Add health indicators and diff views in `AgentsView.vue`, and surface drift warnings on the dashboard attention rail.

## 11. Turn the Dashboard Feed Into Actionable Workflow, Not Just Read-Only Telemetry
**Category:** UX  
**Priority:** Medium  
**Effort:** Medium (days)

**What:** Make `DashboardView.vue` actions executable from the feed: open the suggested task, jump to the relevant note, retry a cron job, or open the responsible repo directly. Reduce the number of “I see the issue, now I still have to navigate somewhere else” moments.
**Why:** The dashboard already synthesizes good signals, especially the broker and attention rail, but it still behaves more like a status board than a control surface. High-value overview screens should shorten the distance to action.
**How:** Add lightweight action buttons to feed items and roster cards, backed by existing task, note, cron, and launcher commands. Keep it intentionally shallow: one-click actions for the top handful of workflows, not a second full app hidden inside the dashboard.

## 12. Add Integration Capability Detection and Graceful Degradation
**Category:** Architecture  
**Priority:** Medium  
**Effort:** Medium (days)

**What:** Teach UMBRA to understand which external capabilities are actually available on the current machine and from the current PM API. Then adapt the UI accordingly instead of exposing half-supported flows.
**Why:** The PM assignment gap already showed the cost of assuming all backends support the same operations. A local-first app that integrates with evolving tools needs runtime capability negotiation, not wishful rendering.
**How:** Add a capability registry in Rust that probes integrations on startup and after config changes. Expose flags like `pm.assignment`, `github.repo_listing`, `tray.available`, or `notifications.available`, and gate buttons, forms, and view affordances based on those flags.

## 13. Build a Local Search Index Across Notes, Tasks, Skills, and Launcher Targets
**Category:** Feature  
**Priority:** Medium  
**Effort:** Medium (days)

**What:** Create one local search surface that can find notes, tasks, skills, agents, and launcher targets from a single query. This should be broader than the command palette and optimized for recall.
**Why:** UMBRA already has many information islands, and the cost of remembering which tab owns which object will only grow. A true local search index is one of the biggest leverage features for an operator-style tool.
**How:** Start with an in-memory index hydrated from existing stores, then persist lightweight normalized records via Rust if needed. Rank results by entity type, recency, and workspace relevance, and route selections back into the correct view with deep links.

## 14. Add Task/Repo/Agent Quick-Link Chips to `NotesView.vue`
**Category:** Feature  
**Priority:** Medium  
**Effort:** Small (hours)

**What:** Let notes embed structured quick-link chips to PM tasks, GitHub repos, agents, and local workspaces instead of relying only on freeform markdown text. These should be insertable from the editor toolbar and clickable in preview.
**Why:** Notes are often the glue between execution systems, but plain markdown makes references lossy and inconsistent. Quick-link chips would make UMBRA notes much more operational without abandoning markdown.
**How:** Add a minimal slash-style insert menu or toolbar actions in `NoteEditor.vue` that drop canonical markdown links or tagged tokens. Resolve those tokens in preview and make them open the relevant task, launcher target, or agent profile.

## 15. Add Screenshot and File Attachments to Notes
**Category:** Feature  
**Priority:** Medium  
**Effort:** Medium (days)

**What:** Support drag-and-drop or paste of screenshots into `NotesView.vue`, storing them in a predictable attachments folder under the vault. Render them in markdown preview and keep paths local.
**Why:** For debugging, UI audits, and planning, screenshots are often more valuable than text. A note system inside a Windows desktop tool feels incomplete if it cannot absorb the user’s native clipboard and file-drop workflow.
**How:** Use Tauri file handling plus frontend paste events to capture images, save them under something like `UMBRA_Notes/_attachments/`, and insert relative markdown links. Keep the first version simple: images only, no attachment manager yet.

## 16. Make the Dashboard Layout User-Configurable
**Category:** UX  
**Priority:** Medium  
**Effort:** Medium (days)

**What:** Let the user hide, reorder, or collapse dashboard cards such as summary stats, activity, deadlines, attention rail, and live roster. Persist those choices per machine.
**Why:** The dashboard is getting richer, but not every panel will matter equally every day. A control plane should bend toward the operator’s habits instead of forcing one canonical layout forever.
**How:** Add a small dashboard settings model to config, with card visibility and order. Keep the interaction lightweight: a “customize dashboard” mode with drag handles or simple move up/down controls, then persist via the existing config store.

## 17. Add Windows 11 Jump Lists for Frequent Actions
**Category:** Feature  
**Priority:** Medium  
**Effort:** Small (hours)

**What:** Expose taskbar Jump List entries such as “Open Tasks”, “New Note”, “Open MMC Repo”, and “Sync PM”. This uses a native Windows affordance that fits UMBRA’s role extremely well.
**Why:** UMBRA is explicitly a Windows 11 desktop app, yet most launch ergonomics still behave like a generic web shell. Jump Lists would make common workflows reachable before the full window is even open.
**How:** Add Windows-specific Tauri integration in the Rust shell layer for taskbar tasks and wire them to app commands or deep links. Start with four or five actions that map cleanly to existing routes and launcher targets.

## 18. Add `umbra://` Deep Links for Cross-Tool Navigation
**Category:** Feature  
**Priority:** Medium  
**Effort:** Medium (days)

**What:** Register a custom URI scheme so Obsidian notes, PM comments, or external tools can open UMBRA directly into a note, task, agent, or launcher target. Make cross-tool navigation first-class.
**Why:** UMBRA already sits in the middle of a broader ecosystem, but navigation into the app still begins at the shell. Deep links would let the rest of the stack treat UMBRA as an addressable system instead of a disconnected desktop window.
**How:** Register `umbra://` with Tauri, define route patterns like `umbra://tasks/{id}` or `umbra://notes/{id}`, and add route handlers in the frontend. Pair it with the command palette and notification center so inbound links always land somewhere meaningful.

## 19. Add End-to-End Coverage for the Real User Flows
**Category:** DX  
**Priority:** Medium  
**Effort:** Medium (days)

**What:** Add end-to-end tests for startup, onboarding, note creation, PM task move, settings persistence, and launcher actions. Unit tests are already decent, but the glue between frontend, Tauri commands, and local files still lacks full-flow coverage.
**Why:** UMBRA’s biggest risks are integration and orchestration regressions, not pure rendering bugs. The more local state, file IO, and external APIs the app touches, the less sufficient unit tests become on their own.
**How:** Start with a small smoke suite around the highest-value flows and mock only the truly external systems. Use the existing test culture in the repo, but focus on cross-layer behavior rather than asserting every pixel.

## 20. Finish the Signed CI Release Pipeline and Staged Updater Rollout
**Category:** DX  
**Priority:** Medium  
**Effort:** Large (weeks)

**What:** Complete the Windows release pipeline with CI-built installers, signing, updater metadata generation, staged rollout, and rollback guidance. UMBRA already has local release mechanics; it now needs a disciplined ship path.
**Why:** Local builds prove feasibility, not release reliability. For a tool that is meant to become the user’s daily console, broken updates or inconsistent installers are trust-destroying.
**How:** Build a GitHub Actions or equivalent Windows pipeline around the existing release scripts, add signing secrets handling, publish updater manifests, and document a staged rollout checklist. Keep release promotion explicit: dev build, candidate, stable.

## Top 5 Quick Wins

1. **#7 Shared async state system** — high UX payoff with relatively little code and no new backend dependency.
2. **#14 quick-link chips in notes** — makes `NotesView.vue` dramatically more operational without changing the storage model.
3. **#17 Windows Jump Lists** — a very Windows-native improvement with small implementation scope and obvious daily utility.
4. **#8 notification center + toasts** — unlocks real-time usefulness from telemetry UMBRA already has.
5. **#3 first-run onboarding** — removes the current “technically works, practically confusing” setup gap.

## Top 3 Architectural Moves

1. **#1 [BREAKING] Rust sync hub** — the single biggest step toward a real local-first orchestration architecture.
2. **#2 [BREAKING] PM domain extraction** — removes the most overloaded view in the app and stabilizes the task board.
3. **#12 capability detection + graceful degradation** — prevents future integrations from turning the UI into a guessing game.
