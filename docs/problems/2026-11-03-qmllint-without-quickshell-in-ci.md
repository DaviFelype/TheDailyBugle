# 2026-11-03 — `qmllint` passa localmente e falha no CI sem Quickshell

- **Status:** resolvido
- **Commits/PRs:** `ci(qml): report qmllint warnings as an annotation` e
  `ci(qml): install quickshell for qmllint` (PR #13)

## Sintoma

O job `ci / qml` falhou só com `Process completed with exit code 255`. Localmente,
`tests/plugin/qmllint.sh` passava, tanto com o Omarchy instalado quanto com o
commit do Omarchy fixado no CI. Os logs do Actions não abrem de dentro do ai-jail
(ver `2026-11-01-actions-logs-blocked-in-ai-jail.md`).

Com o aviso publicado como anotação, apareceu a causa:

```
Warning: BarWidget.qml:2:1: Failed to import Quickshell. Are your import paths set up properly? [import]
Warning: BarWidget.qml:77:3: Process was not found. Did you add all imports and dependencies? [import]
```

## Causa raiz

O container Arch do CI instalava só o `qt6-declarative`. Na Fase 1 o widget
importava apenas `qs.Ui`, e o `qmllint` não conta avisos dentro de módulos
importados. Na Fase 2 o próprio widget passou a importar `Quickshell` e
`Quickshell.Io` (`Process`, `execDetached`), e aí cada uso virou um aviso. Na
máquina local o Quickshell vem com o Omarchy, então o problema não aparecia.

## Solução

- O job instala `quickshell` do mesmo snapshot do Arch Linux Archive.
- `tests/plugin/qmllint.sh`, quando roda no Actions, publica a saída do `qmllint`
  como anotação, que é legível pela API.

## Como evitar

Todo módulo QML importado pelo plugin precisa estar instalado no container do job
`qml`. A anotação deixa a próxima falha legível sem abrir o navegador.
