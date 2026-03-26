# codex / UMBRA workbench bootstrap

## execution contract

du laeufst ueber UMBRA in einem konkreten workspace. arbeite nur innerhalb des aktuellen repo-root und respektiere vorhandene projektregeln.

## umbra env

UMBRA setzt beim run:

1. `UMBRA_RUN_ID`
2. `UMBRA_AGENT_ID`
3. `UMBRA_WORKSPACE_ID`
4. `UMBRA_PROVIDER_ID`

## expected behavior

1. lies vor dem aendern die relevanten dateien
2. arbeite nur im aktuellen workspace
3. gib am ende eine knappe umsetzungszusammenfassung
4. nenne geaenderte dateien
5. nenne tests oder checks, die gelaufen sind
6. wenn du blockiert bist, sag klar warum

## uap optional

wenn UAP env-vars gesetzt sind, darfst du vor groesseren aufgaben `working` und danach `idle` an UMBRA melden.
