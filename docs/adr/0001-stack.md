# ADR-0001 — Stack de desenvolvimento

- **Status:** aceita
- **Data:** 2026-11-01

## Contexto

O TheDailyBugle consome conteúdo não confiável de várias fontes da internet e roda
dentro do bar do Omarchy. Precisa ser seguro, rápido, fácil de empacotar no AUR e
seguir o mesmo modelo do ai-usagebar.

## Decisão

- **Backend em Rust** (edição 2024), com versão fixada em `rust-toolchain.toml`.
- HTTP com `reqwest` + `rustls`; parsing com `serde_json`, `feed-rs` e `quick-xml`;
  CLI com `clap`; config em TOML.
- **Frontend** em QML (Quattro) com a lógica pura em `Model.js`.
- **Testes**: `cargo-nextest`, `wiremock`, `insta`, `cargo-llvm-cov`, `cargo-fuzz`;
  no frontend, `node --test` e `qmllint`.
- **Ferramentas** fixadas em `mise.toml` e instaladas por `scripts/bootstrap.sh`.
- **Regra de supply chain:** só fixar versões publicadas há pelo menos 7 dias.
  Isso vale para ferramentas, crates e GitHub Actions.

## Alternativas

- **Python:** mais rápido de escrever, mas exige runtime na máquina e não gera
  binário único.
- **Go:** viável, mas sem vantagem clara sobre Rust e sem paridade com o ai-usagebar.

## Consequências

- Binário único e memória segura; bom ecossistema de auditoria (`cargo-audit`,
  `cargo-deny`, `cargo-fuzz`).
- Compilação mais lenta que Go e curva de aprendizado maior.
- O fuzzing exige um nightly, fixado em `scripts/bootstrap.sh`.
- A regra dos 7 dias atrasa correções urgentes. Exceção permitida para correções
  de segurança, registrada no PR.
