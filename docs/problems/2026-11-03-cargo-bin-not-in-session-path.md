# 2026-11-03 — Widget não acha o binário instalado com `cargo install`

- **Status:** resolvido
- **Commits/PRs:** `docs(readme): install the binary into ~/.local/bin` (PR #13)

## Sintoma

Teste manual M01 da estratégia 001: o ícone aparece no bar, mas o popup mostra
"Could not load the feed. Is thedailybugle installed and on the PATH?". No
terminal, `thedailybugle feed --json` funciona.

## Causa raiz

O README mandava `cargo install --path . --locked`, que instala em
`~/.cargo/bin`. Esse diretório entra no PATH pelo rc do shell interativo. O
omarchy-shell é iniciado pelo uwsm, com o PATH montado em
`/usr/share/omarchy/default/bash/env-bootstrap`: caminhos do sistema, shims do
mise e `~/.local/bin`, sem `~/.cargo/bin`. O `Process` do widget não acha o
binário, e o widget cai no estado de erro, como previsto no caso M02.

## Solução

Instalar com `cargo install --path . --locked --root ~/.local`, que põe o binário
em `~/.local/bin`, já presente no PATH da sessão (sem novo login).

## Como evitar

O README explica de onde vem o PATH da sessão. O pacote AUR da Fase 6 instala em
`/usr/bin` e elimina o problema para quem não desenvolve o projeto.
