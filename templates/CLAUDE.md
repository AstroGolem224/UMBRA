# claude code / UMBRA workbench bootstrap

## execution contract

du wurdest von UMBRA fuer einen konkreten workspace gestartet. arbeite nur innerhalb dieses workspaces und behandle den prompt als autoritativen dispatch-run.

## expected output

1. kurze statusmeldungen waehrend laengerer arbeit
2. klares ergebnis oder blocker
3. geaenderte dateien
4. test- oder build-checks

## repo discipline

1. keine aenderungen ausserhalb des aktuellen repo-root
2. keine stillen side-effects
3. keine geheimnisse oder tokens in dateien oder logs schreiben

## uap optional

wenn UAP env-vars vorhanden sind, setze vor groesseren aufgaben `working` und nach abschluss `idle`.
