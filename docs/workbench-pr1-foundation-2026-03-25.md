# workbench pr1 foundation - 2026-03-25

## ziel

PR1 baut die technische und UI-grundlage fuer die agent workbench, ohne schon den vollen provider-runtime-stack fertigziehen zu muessen.

der PR1-erfolg ist erreicht, wenn UMBRA:

1. workspaces als echte presets kennt
2. einen workbench composer rendern kann
3. dispatch runs lokal anlegen und anzeigen kann
4. agent- und workspace-auswahl sauber ueber stores und tauri commands verdrahtet
5. spaetere provider-adapter und persona-presets ohne modellbruch aufnehmen kann

## step 0

### was schon existiert und wiederverwendet werden soll

1. agent roster und live-updates aus `src/stores/useAgentStore.ts`
2. agent datenmodell aus `src/interfaces/index.ts`
3. config-persistenz aus `src-tauri/src/commands/config.rs`
4. root- und path-validation aus `src-tauri/src/commands/launcher.rs`
5. UAP task/heartbeat-basis aus `src-tauri/src/uap.rs`

### was PR1 explizit noch nicht loest

1. echte provider-ausfuehrung fuer codex/claude/gemini/kimi
2. per-agent auth-hardening
3. worker-prozessmanagement
4. multi-run streaming logs aus echten provider-cli-prozessen

## recommendation

PR1 soll als **foundation slice** gebaut werden, nicht als halb fertiger end-to-end run.

warum:

1. das UI und datenmodell koennen wir jetzt sauber bauen
2. auth und worker-layer sind die riskanten teile und sollten nicht implizit in denselben diff rutschen
3. das reduziert regressionsrisiko gegen die bestehende agents-view und config-schicht

## scope

### in scope

1. neue route und view fuer die workbench
2. `WorkspacePreset` im frontend und backend
3. `DispatchRun` und `RunEvent` modelle
4. tauri command surface fuer workbench foundation
5. neuer store fuer composer-state, runs und events
6. UI composer mit `message + agent + workspace + mode`
7. run timeline mit lokalen foundation-events
8. nullable hook fuer spaetere `personaId`

### nicht in scope

1. echter folder-picker als default-flow
2. process spawning fuer provider
3. file write execution durch agenten
4. artifact parsing aus echten provider-runs
5. slash commands, channel model, sessions

## file list

### frontend

1. `src/router/index.ts`
2. `src/interfaces/index.ts`
3. `src/stores/useWorkbenchStore.ts`
4. `src/views/WorkbenchView.vue`
5. `src/components/workbench/DispatchComposer.vue`
6. `src/components/workbench/RunTimeline.vue`
7. `src/components/workbench/RunInspector.vue`
8. `src/views/__tests__/WorkbenchView.test.ts`
9. `src/stores/__tests__/useWorkbenchStore.test.ts`

### backend

1. `src-tauri/src/commands/config.rs`
2. `src-tauri/src/commands/workbench.rs`
3. `src-tauri/src/lib.rs`
4. `src-tauri/src/state.rs`
5. `src-tauri/src/types.rs`
6. `src-tauri/src/workbench/mod.rs`
7. `src-tauri/src/workbench/models.rs`
8. `src-tauri/src/workbench/store.rs`
9. `src-tauri/src/workbench/workspaces.rs`
10. `src-tauri/src/workbench/tests.rs`

## datenmodell

### frontend types

```ts
export interface WorkspacePreset {
  id: string
  name: string
  rootPath: string
  writable: boolean
  allowedProviders: string[]
  allowedAgents: string[]
}

export type DispatchMode = "chat" | "task"
export type DispatchStatus = "draft" | "queued" | "working" | "done" | "error" | "cancelled"

export interface DispatchRun {
  id: string
  mode: DispatchMode
  agentId: string
  providerId: string
  workspaceId: string
  prompt: string
  personaId?: string | null
  status: DispatchStatus
  createdAt: string
  updatedAt: string
}

export interface RunEvent {
  id: string
  runId: string
  type: "user_message" | "system" | "agent_message"
  body: string
  createdAt: string
}
```

### backend types

PR1 nutzt dieselben kernfelder in rust. `persona_id` bleibt optional und wird in PR1 noch nicht aktiv befuellt.

## config schema

### neue config-felder

`AppConfig` bekommt:

```text
workspacePresets: WorkspacePreset[]
defaultWorkspaceId?: string
```

### warum in config und nicht sofort sqlite

1. workspace presets sind klar benutzerkonfiguration
2. das passt sauber in die bestehende config-normalisierung
3. dispatch runs selbst duerfen in PR1 bereits in sqlite oder einem workbench-store landen; presets gehoeren trotzdem in die config

## persistence

### recommendation

PR1 soll bereits einen kleinen lokalen workbench-store bekommen, statt runs nur im memory zu halten.

### minimaler sqlite-scope

```text
dispatch_runs
- id
- mode
- agent_id
- provider_id
- workspace_id
- prompt
- persona_id nullable
- status
- created_at
- updated_at

run_events
- id
- run_id
- type
- body
- created_at
```

### rationale

1. run history ist fuer ein workbench-produkt sofort wertvoll
2. foundation-task im backlog nennt sqlite ohnehin explizit
3. wir vermeiden spaeter einen speicher-umbau direkt nach ui-launch

## command surface

### config / presets

1. `list_workspace_presets() -> Vec<WorkspacePreset>`
2. `save_workspace_presets(presets: Vec<WorkspacePreset>) -> Vec<WorkspacePreset>`

### runs

1. `create_dispatch_run(input: CreateDispatchRunInput) -> DispatchRun`
2. `list_dispatch_runs() -> Vec<DispatchRun>`
3. `get_dispatch_run(run_id: String) -> Option<DispatchRun>`
4. `list_run_events(run_id: String) -> Vec<RunEvent>`
5. `cancel_dispatch_run(run_id: String) -> DispatchRun`

### debug / foundation helper

1. `append_run_event(input: AppendRunEventInput) -> RunEvent`

das helper-command ist fuer PR1 okay, damit UI und store mit echten event-flows entwickelt und getestet werden koennen, auch bevor der worker fertig ist.

## tauri event contract

PR1 soll bereits inkrementelle events sprechen.

```text
workbench:run-created
workbench:run-updated
workbench:event-added
```

payloads sollen vollstaendige `DispatchRun`- oder `RunEvent`-objekte sein, keine diff-fragmente.

## ui contract

### composer layout

```text
+--------------------------------------------------------------+
| message textarea                                             |
+----------------------+----------------------+-----------------+
| agent dropdown       | workspace dropdown   | mode toggle     |
+----------------------+----------------------+-----------------+
| persona slot (empty in pr1, reserverter platz)               |
+--------------------------------------------------------------+
| send button                                                 |
+--------------------------------------------------------------+
```

### detailregeln

1. send ist disabled ohne `prompt + agent + workspace`
2. provider wird in PR1 aus agent-meta oder default mapping abgeleitet, noch nicht frei separat gewaehlt
3. persona-slot bleibt als layout-hook vorhanden, aber kann in PR1 hidden oder disabled sein
4. advanced folder picker erscheint in PR1 nicht

## provider mapping in PR1

PR1 fuehrt noch keinen vollstaendigen provider-adapter ein, braucht aber ein minimales mapping:

```text
agentId -> providerId
codex-* -> codex
claude-* -> claude
gemini-* -> gemini
kimi-* -> kimi
fallback -> custom
```

das kann spaeter in die echte adapter-registry umziehen. fuer PR1 reicht eine explizite helper-funktion.

## workspace validation

die validierung soll code aus `launcher.rs` wiederverwenden oder in eine gemeinsame helper-stelle ziehen.

### regeln

1. root path darf nicht leer sein
2. path muss existieren
3. path wird canonicalized
4. preset-name ist required
5. `allowedAgents` darf leer sein, bedeutet dann `alle`

## run lifecycle in PR1

```text
draft
-> queued
-> working     (nur ueber debug/helper oder test flow)
-> done|error|cancelled
```

PR1 braucht keinen echten worker, aber der statusgraph soll bereits final sein, damit spaetere runtime-arbeit nicht die UI neu schneiden muss.

## persona hook

der neue backlog-task `Persona Dropdown fuer den Workbench Composer` bleibt nach PR1 ein follow-up, aber PR1 muss zwei hooks vorbereiten:

1. optionales `personaId` im run-modell
2. reservierter composer-slot zwischen workspace und submit

### warum nicht voll in PR1

1. personas ohne provider-runtime sind schnell nur dekorative labels
2. wir brauchen zuerst die workbench-grundform
3. der hook kostet fast nichts und verhindert spaetere migrationsarbeit

## testplan

### rust

1. workspace preset validation akzeptiert gueltige pfade
2. workspace preset validation blockt leere oder traversal-pfade
3. create_dispatch_run persistiert run + erstes user event
4. cancel_dispatch_run setzt status korrekt
5. list_run_events filtert pro run

### vue

1. composer disabled state ohne pflichtfelder
2. composer send mit agent + workspace + text
3. timeline rendert user message und system events
4. persona-slot zerlegt layout nicht

### integration

1. tauri event `workbench:run-created` aktualisiert den store
2. tauri event `workbench:event-added` aktualisiert timeline inkrementell

## failure modes

1. workspace preset zeigt auf geloeschten ordner
   - handling: preset rendern, aber send blocken
   - test: ja
2. agent ist offline
   - handling: send erlauben oder blocken ist produktentscheidung; empfehlung in PR1: blocken mit sichtbarer meldung
   - test: ja
3. run wird angelegt, erstes event aber nicht persistiert
   - handling: transaction erforderlich
   - test: ja
4. provider-mapping unklar
   - handling: klarer `unsupported provider` status statt stiller queue
   - test: ja

## konkrete PR1 reihenfolge

1. rust modelle + store + commands
2. config schema fuer workspace presets
3. frontend types + store
4. workbench route + shell view
5. composer + timeline + inspector
6. tests fuer store und view

## naechster PR nach PR1

nach PR1 sollte direkt der worker/adapter-slice kommen:

1. `AgentAdapter` trait
2. codex/claude/gemini spawn specs
3. echte run execution
4. auth-hardening

