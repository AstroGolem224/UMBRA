# agent setup guide - 2026-03-25

## ziel

UMBRA kann workbench-runs jetzt direkt fuer `codex`, `claude` und `gemini` starten. der produktive pfad ist damit:

1. provider-command in `settings -> workbench providers` setzen
2. workspace preset in UMBRA anlegen
3. repo-instruktionsdatei fuer den jeweiligen provider hinterlegen
4. run aus der workbench dispatchen

fuer den agent-roster und pending-task-polling bleibt UAP optional daneben bestehen.

## aktueller runtime-schnitt

### workbench dispatch

UMBRA startet den provider selbst im gewaehlten workspace.

```text
Workbench composer
-> create_dispatch_run
-> workbench runner
-> provider command in selected workspace cwd
-> stdout/stderr stream into timeline
-> run status done/error
```

### optionaler listener / roster-modus

wenn ein agent zusaetzlich im agent-roster sichtbar sein und UAP-pending-tasks pollen soll, kann er weiter heartbeats an den UAP-server senden. das ist von der workbench-dispatch-logik getrennt.

## provider commands in UMBRA

in `Settings > workbench providers` gibt es jetzt pro provider genau ein feld:

1. `codex`
2. `claude`
3. `gemini`
4. `kimi`

UMBRA nutzt diese reihenfolge:

1. provider-command aus den settings
2. env override `UMBRA_<PROVIDER>_COMMAND`
3. built-in default command

beispiele:

```text
codex  -> codex
claude -> claude
gemini -> gemini
```

### windows-hinweis fuer codex

auf dem aktuellen windows-setup ist die WindowsApps-Codex-binary als child-process nicht zuverlaessig startbar und antwortet lokal mit `access denied`. deshalb sollte fuer windows entweder

1. ein expliziter launchbarer `codex`-pfad konfiguriert werden oder
2. `wsl.exe` mit funktionierendem Codex in WSL verfuegbar sein

wenn beides nicht klappt, zeigt UMBRA den fehler sichtbar im run statt still zu scheitern.

## offizielle non-interactive launch-pfade

UMBRA nutzt fuer wave 1 diese provider-flows:

1. `codex exec "<prompt>"`
2. `claude -p "<prompt>" --output-format stream-json`
3. `gemini -p "<prompt>" --output-format stream-json`

quellen:

1. OpenAI: [Unlocking the Codex harness](https://openai.com/index/unlocking-the-codex-harness/)
2. Anthropic: [Claude Code CLI reference](https://docs.anthropic.com/en/docs/claude-code/cli-reference)
3. Google: [Gemini CLI README](https://github.com/google-gemini/gemini-cli)

## repo instruction surfaces

### codex

`AGENTS.md`

### claude

`CLAUDE.md`

### gemini

`GEMINI.md`

im repo liegen vorbereitete templates unter:

1. `templates/AGENTS.codex.md`
2. `templates/CLAUDE.md`
3. `templates/GEMINI.md`
4. `templates/worker.env.example`

## minimaler setup pro provider

### codex

1. codex cli so installieren, dass der command wirklich als child-process startbar ist
2. in UMBRA `Settings > workbench providers > codex` setzen
3. `templates/AGENTS.codex.md` nach `AGENTS.md` ins repo uebernehmen oder anpassen
4. workspace preset mit `writable = true` anlegen

### claude

1. Claude Code installieren und `claude` im terminal pruefen
2. in UMBRA `Settings > workbench providers > claude` setzen
3. `templates/CLAUDE.md` nach `CLAUDE.md` ins repo uebernehmen oder anpassen
4. authentifizierung fuer Claude Code ausserhalb von UMBRA einmal sauber abschliessen

### gemini

1. Gemini CLI installieren und `gemini -p "ping"` lokal pruefen
2. in UMBRA `Settings > workbench providers > gemini` setzen
3. `templates/GEMINI.md` nach `GEMINI.md` ins repo uebernehmen oder anpassen
4. auth per Google Login, `GEMINI_API_KEY` oder Vertex AI konfigurieren

## UAP listener / heartbeat optional

wenn ein provider zusaetzlich im roster auftauchen und auf UAP-pending-tasks reagieren soll:

1. `templates/worker.env.example` kopieren und env-vars setzen
2. agent auth token aus `Settings > Agent Auth Tokens` kopieren
3. heartbeat-hook in die jeweilige provider-konfiguration uebernehmen
4. statuswechsel `working` / `idle` an UAP melden

### per-agent auth tokens

jeder agent authentifiziert sich ueber seinen eigenen `X-Agent-Token` header. tokens werden automatisch generiert und koennen in `Settings > Agent Auth Tokens` eingesehen, kopiert und rotiert werden.

```bash
curl -X POST http://127.0.0.1:8765/api/agents/forge/heartbeat \
  -H "Content-Type: application/json" \
  -H "X-Agent-Token: <token-aus-settings>" \
  -d '{"name":"Forge","status":"working","activeTaskId":"current-task"}'
```

das ist fuer workbench-dispatch **nicht** erforderlich. workbench-runs funktionieren ohne dauerhaften listener, weil UMBRA den provider on-demand startet.

## empfohlene env-vars

```text
UMBRA_AGENT_ID
UMBRA_AGENT_NAME
UMBRA_AGENT_ROLE
UMBRA_WORKSPACE_ID
UMBRA_RUN_ID
UMBRA_CODEX_COMMAND
UMBRA_CLAUDE_COMMAND
UMBRA_GEMINI_COMMAND
```

## bekannte grenzen in diesem stand

1. cancel stoppt aktuell den run-status, killt aber noch nicht jeden provider-prozess hart
2. codex auf windows kann je nach install-art einen expliziten launch-path oder WSL brauchen
3. kimi ist im settings-surface vorbereitet, aber noch nicht als wave-1-runner verdrahtet

## acceptance check

ein provider-setup gilt erst als fertig, wenn diese drei checks lokal gruen sind:

1. provider-command startet ausserhalb von UMBRA im terminal
2. ein UMBRA-workbench-run erreicht `working` und streamt stdout/stderr in die timeline
3. der provider kann im gewaelten workspace dateien lesen und bei task-mode auch schreiben
