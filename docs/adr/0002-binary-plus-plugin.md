# ADR-0002 — Arquitetura: binário Rust + plugin QML

- **Status:** aceita
- **Data:** 2026-11-01

## Contexto

Plugins do Quattro são QML. O QML consegue fazer requisições de rede, mas é difícil
de testar, não tem fuzzing e renderiza rich text com facilidade. As fontes do feed
(CVEs, sites de vazamento de ransomware, RSS) trazem texto controlado por terceiros,
inclusive por atacantes.

## Decisão

Separar em duas partes, como o ai-usagebar:

1. **Binário `thedailybugle`** (Rust): busca as fontes, valida, sanitiza, faz cache
   e imprime um JSON em stdout (`thedailybugle feed --json`). O formato desse JSON é
   um contrato versionado, validado por JSON Schema.
2. **Plugin QML** `davifelype.thedailybugle`: só executa o binário e exibe o
   resultado. A lógica de exibição fica em `Model.js` para ser testada com `node`.

Regras da fronteira:

- No JSON, todo texto vem sem marcação e todo link é `https://`.
- O QML usa `Text.PlainText` e abre links só via `xdg-open`, sem shell.
- Qualquer falha do binário vira um estado de erro no widget, nunca um crash do bar.

## Consequências

- Toda a parte sensível fica num lugar testável e fuzzável.
- Duas peças para empacotar: binário no AUR e plugin via `omarchy plugin add`.
- O plugin depende do binário instalado e de uma versão compatível do contrato
  JSON. O JSON leva um campo de versão para detectar incompatibilidades.
