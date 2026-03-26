# phase 8: ui/workspace pass

## ceo/ui review

### workbench
1. der alte composer war funktional, aber zu technisch und ohne klare priorisierung.
2. die wichtigste information, die der user sucht, war nicht prominent genug: die letzte agent-antwort.
3. workspace war sichtbar, aber nicht als echte arbeitsgrenze erlebbar. es fehlten direkte folder/terminal-actions.

### ops room
1. der alte ops-room war ein stack aus formularen, aber kein professioneller operations-screen.
2. routing war nur implizit über mentions/defaults/sessions steuerbar. ein explizites `agent antwortet`-feld fehlte.
3. workspace-kontext war zwar im datamodell da, aber nicht als nutzbare aktion in der ui. das erklärt den wahrgenommenen bruch rund um `launch target not found: workspace`.

## in phase 8 umgesetzt
1. native workspace-actions über den workbench-command-layer:
   - `open_workspace_folder`
   - `open_workspace_terminal`
2. ops-room composer mit explizitem response target:
   - neues feld `agent antwortet`
   - backend-routing priorisiert jetzt explizit gesetztes `agentId`
3. workbench ui neu geschnitten:
   - klarerer composer
   - prominente `agent answer`-karte
   - direkte workspace-actions
   - professionellere status- und artifact-darstellung
4. ops-room ui neu geschnitten:
   - channel-banner mit workspace-kontext
   - composer mit `agent antwortet`
   - workspace-card in der rail
   - klarere jobs/approvals/rules/sessions sections

## fehlende funktionen nach phase 8
1. echter workspace-manager außerhalb von settings:
   - presets direkt aus workbench/ops room anlegen
   - invalid workspace/grant-konflikte inline heilen
2. ops-room reply ergonomics:
   - echte inline replies auf einzelne messages
   - sichtbarer parent-thread statt nur meta-labels
3. provider/präsenz tiefer in ops room:
   - agent tippt / arbeitet / blockiert
   - live run-state pro channel ohne timeline lesen zu müssen
4. result drilldown:
   - changed files diff preview
   - tests als eigene inspector-surface
5. workspace onboarding:
   - bootstrap/checklist schneller aus workbench oder ops room statt nur settings

## nächste phasen

### phase 9: workflow completion
1. workspace manager inline
2. threaded replies + reply composer
3. richer run/job result inspector

### phase 10: operational visibility
1. provider presence in ops room
2. live run badges per channel/job
3. blocked/needs-input escalation ui

### phase 11: polish + onboarding
1. bootstrap/checklist surfacing in workbench
2. empty states, first-run guidance, better failure copy
3. deeper browser e2e for workspace and reply flows
