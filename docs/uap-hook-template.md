# UAP — UMBRA Agent Protocol Hook Template

UMBRA läuft auf `100.98.137.48:8765` (Tailscale). Der Server akzeptiert Verbindungen von
allen Interfaces (localhost + Tailscale-Netz). Offline-Timeout: **30 Minuten**.

---

## settings.local.json — PostToolUse Hook

Agenten-ID und Name über Umgebungsvariablen konfigurierbar.
Standard: `forge` / `Forge` falls nicht gesetzt.

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash -c 'AGENT_ID=${UMBRA_AGENT_ID:-forge}; AGENT_NAME=${UMBRA_AGENT_NAME:-Forge}; AGENT_ROLE=${UMBRA_AGENT_ROLE:-Code Agent}; curl -s --connect-timeout 3 -X POST http://100.98.137.48:8765/api/agents/$AGENT_ID/heartbeat -H \"Content-Type: application/json\" -H \"X-Agent-Token: umbra-uap-2026\" -d \"{\\\"name\\\":\\\"$AGENT_NAME\\\",\\\"role\\\":\\\"$AGENT_ROLE\\\",\\\"status\\\":\\\"online\\\"}\" > /dev/null 2>&1 || true'"
          }
        ]
      }
    ]
  }
}
```

Umgebungsvariablen in der Shell setzen (z.B. in `.bashrc` / `.zshrc`):

```bash
export UMBRA_AGENT_ID="forge"
export UMBRA_AGENT_NAME="Forge"
export UMBRA_AGENT_ROLE="Web / Code Agent"
```

---

## Heartbeat API

```
POST http://100.98.137.48:8765/api/agents/:id/heartbeat
X-Agent-Token: umbra-uap-2026
Content-Type: application/json

{
  "name": "Forge",
  "role": "Web / Code Agent",
  "status": "online",
  "allowedTools": ["read", "write", "edit", "bash", "glob", "grep"],
  "skills": ["TypeScript", "Vue 3", "Rust", "Git", "Architecture"],
  "activeTaskId": null
}
```

Response — enthält ausstehende Tasks aus der UMBRA-Queue:

```json
{
  "ok": true,
  "pendingTasks": [
    {
      "id": "...",
      "title": "Review PR #42",
      "description": "Check for security issues",
      "priority": "high"
    }
  ]
}
```

---

## CLAUDE.md — Automatische Registrierung beim Sitzungsstart

```markdown
## UMBRA Agent Protocol (UAP)

Beim Start jeder Sitzung: Heartbeat senden und pending Tasks prüfen.

```bash
AGENT_ID="${UMBRA_AGENT_ID:-forge}"
AGENT_NAME="${UMBRA_AGENT_NAME:-Forge}"
AGENT_ROLE="${UMBRA_AGENT_ROLE:-Code Agent}"

RESPONSE=$(curl -s --connect-timeout 3 \
  -X POST http://100.98.137.48:8765/api/agents/$AGENT_ID/heartbeat \
  -H "Content-Type: application/json" \
  -H "X-Agent-Token: umbra-uap-2026" \
  -d "{\"name\":\"$AGENT_NAME\",\"role\":\"$AGENT_ROLE\",\"status\":\"online\"}")

# Pending Tasks ausgeben falls vorhanden
echo "$RESPONSE" | grep -o '"title":"[^"]*"' | sed 's/"title":"//;s/"//'
```

- Wenn UMBRA nicht erreichbar: stillschweigend fortfahren (`|| true`)
- Offline-Timeout: 30 Minuten ohne Heartbeat → Status wird auf "offline" gesetzt
- Token: `umbra-uap-2026` — niemals in Logs oder Commits
```

---

## Working / Idle Status Protocol

Agenten **müssen** ihren Status vor und nach jeder Aufgabe explizit melden.

### Vor Beginn einer Aufgabe → `working`

```bash
curl -s --connect-timeout 3 -X POST http://100.98.137.48:8765/api/agents/${UMBRA_AGENT_ID:-forge}/heartbeat \
  -H "Content-Type: application/json" \
  -H "X-Agent-Token: umbra-uap-2026" \
  -d "{\"name\":\"${UMBRA_AGENT_NAME:-Forge}\",\"status\":\"working\",\"activeTaskId\":\"<kurze Beschreibung oder Task-ID>\"}" > /dev/null 2>&1 || true
```

### Nach Abschluss einer Aufgabe → `idle`

```bash
curl -s --connect-timeout 3 -X POST http://100.98.137.48:8765/api/agents/${UMBRA_AGENT_ID:-forge}/heartbeat \
  -H "Content-Type: application/json" \
  -H "X-Agent-Token: umbra-uap-2026" \
  -d "{\"name\":\"${UMBRA_AGENT_NAME:-Forge}\",\"status\":\"idle\",\"activeTaskId\":null}" > /dev/null 2>&1 || true
```

### CLAUDE.md Instruktion (generisch — für jeden Agenten)

Füge folgenden Block in die CLAUDE.md des jeweiligen Agenten ein:

```markdown
## UMBRA Agent Protocol (UAP) — Status

Vor jeder nicht-trivialen Aufgabe (>5 Zeilen Code, Recherche, Planung):
- Status auf `working` setzen: `POST .../heartbeat` mit `{"status":"working","activeTaskId":"<kurze Beschreibung>"}`

Nach Abschluss der Aufgabe (oder bei Sitzungsende):
- Status auf `idle` setzen: `POST .../heartbeat` mit `{"status":"idle","activeTaskId":null}`

Endpoint: `http://100.98.137.48:8765/api/agents/<agent-id>/heartbeat`
Token: `X-Agent-Token: umbra-uap-2026`
Variablen: `UMBRA_AGENT_ID`, `UMBRA_AGENT_NAME`, `UMBRA_AGENT_ROLE`
```

### Status-Werte (vollständig)

| Status | Bedeutung | Visuell |
|--------|-----------|---------|
| `online` | Heartbeat aktiv, keine spezifische Aufgabe | Grün, pulsierend |
| `working` | Bearbeitet gerade eine Aufgabe | Amber, Atemanimation |
| `idle` | Eingeloggt, wartet auf Aufgaben | Cyan, statisch |
| `offline` | Kein Heartbeat seit >30 Min | Grau, statisch |
| `error` | Verbindungs- oder Ausführungsfehler | Rot, statisch |

---

## Bekannte Agent-IDs

| Agent | ID      | Rolle                        |
|-------|---------|------------------------------|
| Forge | `forge` | Web / Code Agent             |
| Prism | `prism` | Godot / Game Dev Agent       |
| Jim   | `jim`   | Master Dev / Architecture    |

Neue Agenten werden beim ersten Heartbeat automatisch in der Registry angelegt.
