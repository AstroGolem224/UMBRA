# agent workbench spec - 2026-03-25

## tl;dr

UMBRA soll fuer dieses feature keinen generischen chat-server nachbauen. der richtige v1-schnitt ist eine `agent workbench`: ein composer mit `message + agent + workspace`, eine timeline fuer status und rueckmeldungen, und ein gemeinsamer dispatch-contract fuer `codex`, `claude`, `gemini` und spaeter `kimi`.

das ist kein neues parallelprodukt neben UMBRAs agent-roster. es ist die naechste stufe der bestehenden UAP- und launcher-architektur.

## produktthese

wenn der user einem agenten in UMBRA arbeit geben will, braucht er in der regel vier dinge:

1. eine klare nachricht oder aufgabe
2. einen konkreten agenten bzw. provider
3. einen konkreten workspace, in dem gelesen und geschrieben werden darf
4. sichtbarkeit, was der agent gerade tut, aendert und testet

ein nacktes chatfenster loest nur punkt 1. UMBRA muss deshalb eine `dispatch console` bauen, nicht nur einen messenger.

## step 0 - scope check

### was schon existiert

1. UAP heartbeats und task-polling existieren bereits in `src-tauri/src/uap.rs`.
2. agent roster, status und task push existieren bereits in `src/views/AgentsView.vue`, `src/stores/useAgentStore.ts` und `src/interfaces/index.ts`.
3. repo-root-basierte folder-guards existieren bereits in `src-tauri/src/commands/launcher.rs`.
4. config-persistenz fuer `repoRootPath`, `customAgents` und UAP-settings existiert bereits in `src-tauri/src/commands/config.rs`.
5. ein erstes agent-hook-template existiert bereits in `docs/uap-hook-template.md`.

### minimum fuer den kern

1. neue workbench-view mit composer und timeline
2. workspace presets statt freier pfadwahl als standard
3. provider-unabhaengiger dispatch-contract
4. worker/bootstrap-layer fuer codex, claude und gemini
5. per-agent identity statt globalem UAP-token fuer schreibfaehige workspaces

### was wir bewusst noch nicht brauchen

1. multi-channel-slack-artige chatrooms
2. automatische agent-zu-agent-gespraeche
3. session-engine und turn-taking
4. freie ordnerwahl als standard-flow
5. native Kimi-CLI auf windows

## produktschnitt fuer v1

### user flow

```text
user waehlt workspace preset
-> user waehlt agent/provider
-> user schreibt message oder task
-> UMBRA erstellt dispatch run
-> worker startet provider im gewaehlten workspace
-> provider arbeitet lokal im workspace
-> UMBRA zeigt status, logs, dateiaenderungen und result
```

### UI contract

die workbench-view braucht in v1 genau diese oberflaechen:

1. `composer`
   - grosses textfeld
   - dropdown `agent`
   - dropdown `workspace`
   - toggle `chat` oder `task`
   - optional `advanced` fuer freien folder-picker
2. `run timeline`
   - user message
   - system events wie `queued`, `working`, `waiting for input`, `done`, `error`
   - agent antworten
   - file summary und test summary
3. `right rail`
   - workspace-details
   - provider-capabilities
   - quick actions: `open folder`, `open terminal`, `re-run`, `cancel`

### workflow-entscheidung

`chat` und `task zuschieben` bleiben dieselbe grundaktion. der einzige unterschied ist der contract:

1. `chat` erwartet vor allem textantworten
2. `task` erwartet einen ausfuehrungsversuch mit dateiaenderungen, tests und status

## architektur

### system flow

```text
WorkbenchView
  -> tauri command: create_dispatch_run
  -> workbench service
  -> workspace lease validation
  -> adapter resolution
  -> worker spawn
  -> provider cli/api run in workspace cwd
  -> run events persisted
  -> tauri events pushed back into UI
```

### target modules

#### frontend

1. `src/views/WorkbenchView.vue`
2. `src/components/workbench/DispatchComposer.vue`
3. `src/components/workbench/RunTimeline.vue`
4. `src/components/workbench/RunInspector.vue`
5. `src/stores/useWorkbenchStore.ts`

#### backend

1. `src-tauri/src/commands/workbench.rs`
2. `src-tauri/src/workbench/mod.rs`
3. `src-tauri/src/workbench/models.rs`
4. `src-tauri/src/workbench/workspaces.rs`
5. `src-tauri/src/workbench/dispatch.rs`
6. `src-tauri/src/workbench/adapters.rs`
7. `src-tauri/src/workbench/worker.rs`

### minimal datenmodell

```text
WorkspacePreset
- id
- name
- rootPath
- defaultBranch?
- writable
- allowedProviders[]
- allowedAgents[]

DispatchRun
- id
- mode(chat|task)
- agentId
- providerId
- workspaceId
- cwd
- prompt
- status(queued|working|waiting|done|error|cancelled)
- createdAt
- updatedAt

RunEvent
- id
- runId
- type(user_message|system|stdout|stderr|agent_message|artifact)
- body
- createdAt

RunArtifact
- id
- runId
- kind(file|test|diff|link)
- label
- value
```

### warum diese grenze

1. `WorkspacePreset` ist die produktgrenze fuer sicherheit und usability.
2. `DispatchRun` ist die echte einheit der arbeit, nicht eine lose chat-message.
3. `RunEvent` und `RunArtifact` geben genug timeline-fidelity fuer v1, ohne direkt das volle ops-room-channelmodell zu erzwingen.

## workspace-modell

### standard

UMBRA soll standardmaessig nur in gespeicherte workspace presets dispatchen. ein preset zeigt auf einen echten lokalen ordner und beschreibt:

1. wo gearbeitet werden darf
2. welche provider dort erlaubt sind
3. ob der workspace schreibbar ist

### advanced mode

ein freier folder-picker ist erlaubt, aber nur:

1. hinter einem expliziten `advanced`-toggle
2. nach canonicalize + root-check
3. mit sichtbarem warning state

### sicherheitsregeln

1. keine shell-eingaben aus dem prompt ableiten
2. workspace-pfade immer canonicalizen
3. workspace leases nur fuer den gewaehllten run ausstellen
4. schreibzugriff nur fuer explizit freigegebene workspaces
5. keine provider-ausfuehrung ohne gueltige agent identity

## gemeinsamer adapter-contract

alle provider muessen gegen denselben UMBRA-contract laufen. die UI darf nicht wissen, wie `codex`, `claude`, `gemini` oder `kimi` intern gestartet werden.

### trait-skizze

```rust
pub trait AgentAdapter {
    fn provider_id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn supports_windows(&self) -> bool;
    fn supports_streaming(&self) -> bool;
    fn instruction_surface(&self) -> &'static str;
    fn build_spawn_spec(
        &self,
        run: &DispatchRun,
        workspace: &WorkspacePreset,
        auth: &AgentAuth,
    ) -> Result<SpawnSpec, WorkbenchError>;
}
```

### spawn spec

```text
SpawnSpec
- command
- args[]
- cwd
- env{}
- stdio mode
- optional bootstrap files to ensure
```

### designregel

der adapter baut nur den provider-spezifischen startplan. das polling, status-reporting, timeout-handling und artifact-parsing gehoert in den gemeinsamen worker.

## provider-matrix

### codex

1. `codex` ist tier-1 fuer wave 1.
2. instructions-surface ist primaer `AGENTS.md`.
3. der adapter startet codex immer mit dem gewaehlten workspace als `cwd`.
4. UMBRA stellt env-vars wie `UMBRA_AGENT_ID`, `UMBRA_WORKSPACE_ID` und `UMBRA_RUN_ID`.
5. der bootstrap-flow soll ein repo-lokales `AGENTS.md` respektieren und optional ein UMBRA-spezifisches appendix-file einblenden.

### claude

1. `claude` ist tier-1 fuer wave 1.
2. instructions-surface ist `CLAUDE.md`.
3. existing hooks koennen fuer heartbeat/status genutzt werden, aber UMBRA soll nicht von hooks allein abhaengen.
4. auch fuer claude geht der echte run ueber denselben worker-contract.

### gemini

1. `gemini` ist tier-1 fuer wave 1.
2. instructions-surface ist `GEMINI.md`.
3. keine undokumentierten hook-erwartungen in den kern schreiben.
4. gemini wird ueber wrapper/headless invocation in denselben dispatch-contract eingebunden.

### kimi

1. `kimi` bleibt produktseitig wichtig.
2. fuer windows-first rollout wird Kimi zunaechst ueber adapter/api-kompatibilitaet geplant.
3. native Kimi-CLI kommt erst, wenn der windows-pfad tragfaehig ist.

## worker/bootstrap-layer

### ziel

statt provider direkt aus dem UI oder per zufaelligen shell-skripten zu starten, fuehrt UMBRA einen kleinen worker ein, zum beispiel `umbra-agent-worker`.

### worker-flow

```text
create run
-> validate workspace
-> resolve adapter
-> worker sends heartbeat: working
-> worker ensures instruction files / bootstrap env
-> worker spawns provider in workspace cwd
-> worker captures output and provider result
-> worker posts timeline events and artifacts
-> worker sends heartbeat: idle or error
```

### warum ein worker statt direktem CLI spawn

1. gemeinsame statuslogik
2. einheitliches timeout- und cancel-verhalten
3. provider-spezifische details bleiben gekapselt
4. die setup-anleitung wird fuer alle agenten einfacher

## codex bootstrap

### v1-annahme

codex wird ueber denselben worker wie die anderen provider gestartet, bekommt aber eine explizite `AGENTS.md`-surface.

### setup-regeln

1. workspace hat ein repo-lokales `AGENTS.md` fuer projektregeln
2. UMBRA kann optional ein `AGENTS.umbra.md` oder aehnliches appendix-file erzeugen
3. der worker setzt:
   - `UMBRA_AGENT_ID`
   - `UMBRA_AGENT_NAME`
   - `UMBRA_WORKSPACE_ID`
   - `UMBRA_RUN_ID`
   - `UMBRA_UAP_BASE_URL`
4. wenn kein `AGENTS.md` vorhanden ist, bietet UMBRA einen bootstrap-button an

### codex-specific acceptance criteria

1. codex startet im korrekten workspace
2. codex respektiert vorhandene repo-instruktionen
3. codex kann dateien lesen und schreiben
4. run-timeline zeigt status und result sauber an
5. fehler im bootstrap sind fuer den user sichtbar

## setup-guide deliverables

die docs-task soll nicht nur prose liefern, sondern echte setup-assets.

### docs

1. `docs/agent-setup-guide-YYYY-MM-DD.md`
2. `docs/workbench-provider-matrix-YYYY-MM-DD.md`
3. `docs/uap-worker-bootstrap-YYYY-MM-DD.md`

### templates

1. `templates/AGENTS.codex.md`
2. `templates/CLAUDE.md`
3. `templates/GEMINI.md`
4. `templates/worker.env.example`

### scripts oder binaries

1. `scripts/umbra-agent-worker.ps1` fuer den ersten bootstrap oder
2. spaeter ein nativer rust-worker unter `src-tauri/bin/`

## auth und trust boundaries

das ist der haerteste technische blocker.

### was heute nicht reicht

das aktuelle UAP-modell mit einem globalen `x-agent-token` reicht fuer telemetrie, aber nicht fuer workspace-schreibzugriff.

### benoetigt fuer v1

1. per-agent identity oder signierte registration
2. provider-session an run + workspace binden
3. keine schreibfaehigen workspaces ohne gueltige identity
4. klare visible errors statt silent drop

## rollout

### wave 1 - dispatch mvp

1. workbench view mit composer und timeline
2. workspace presets
3. codex, claude und gemini als tier-1 adapter
4. worker-bootstrap fuer task dispatch
5. run status events in UI

### wave 2 - hardening

1. per-agent auth
2. persisted run history
3. cancel / retry / resume
4. file and test artifacts

### wave 3 - expansion

1. kimi adapter fuer windows-safe path
2. advanced folder picker
3. channel/job-modell als ops-room-erweiterung
4. spaeter session-engine und multi-agent orchestration

## testmatrix

```text
new path                                test type
workspace preset validation             rust unit
advanced folder canonicalization        rust unit
adapter spawn spec generation           rust unit
worker status lifecycle                 rust integration
run -> timeline event fanout            rust + store tests
composer send flow                      vue component test
workspace + agent selection             vue store/view test
codex bootstrap fallback                rust unit
provider unsupported on windows         rust unit
clear error on missing bootstrap        e2e or integration
```

### failure modes

1. workspace path zeigt ausserhalb des erlaubten roots
   - test: ja
   - handling: reject before spawn
   - user feedback: klarer error
2. provider nicht installiert
   - test: ja
   - handling: spawn error
   - user feedback: klarer error
3. heartbeat/auth ungueltig
   - test: ja
   - handling: run blocked
   - user feedback: klarer error
4. provider laeuft, aber schreibt keine verwertbare rueckmeldung
   - test: teilweise
   - handling: timeline endet mit incomplete result
   - user feedback: muss sichtbar bleiben, kein silent success

## nicht in scope

1. slack-artige channels fuer alle agenten gleichzeitig
2. automatische agent-zu-agent-gespraeche
3. voice, image uploads, attachments
4. freie folder-pfade als standard-flow
5. provider-spezifische sonder-ui pro agent
6. native Kimi-CLI auf windows in wave 1

## klare empfehlung

1. diese anforderung als `dispatch-first` mvp innerhalb des ops-room-epics bauen
2. `codex`, `claude` und `gemini` als first-class provider behandeln
3. `kimi` bewusst vorbereiten, aber nicht den v1-kritischen pfad davon abhaengig machen
4. workspaces als produktgrenze ernst nehmen, nicht als nachtraeglichen security-fix
5. den worker als gemeinsame schicht einfuehren, bevor vier provider separat verdrahtet werden

## externe referenzen

1. Claude Code overview: https://docs.anthropic.com/en/docs/claude-code/overview
2. Gemini CLI repository: https://github.com/google-gemini/gemini-cli
3. Kimi programming tools / agent support: https://platform.moonshot.ai/docs/guide/agent-support
4. Kimi CLI support: https://platform.moonshot.ai/docs/guide/kimi-cli-support
