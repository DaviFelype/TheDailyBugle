# 2026-11-01 — `cargo llvm-cov` trava sem mensagem

- **Status:** resolvido (falta rodar o bootstrap no host)
- **Commits/PRs:** `fix` em `scripts/bootstrap.sh` (branch `ci/pipeline`)

## Sintoma

`cargo llvm-cov nextest --summary-only` ficou mais de 2 minutos parado, sem saída,
depois de `cargo nextest run` passar normalmente.

## Causa raiz

O componente `llvm-tools-preview` não estava instalado no toolchain 1.99.1. O
`rust-toolchain.toml` lista o componente, mas o toolchain já existia (instalado pelo
mise) e o `rustup toolchain install` não adicionou os componentes que faltavam. O
`cargo llvm-cov` então pergunta se pode instalar e espera uma resposta no terminal,
que dentro do agente nunca chega. No ai-jail, `~/.rustup` é só leitura, então nem
daria para instalar de lá.

## Solução

`scripts/bootstrap.sh` roda `rustup component add clippy rustfmt llvm-tools-preview`
explicitamente. No CI, o toolchain é instalado do zero a partir do
`rust-toolchain.toml`, com os componentes.

## Como evitar

O bootstrap imprime `cargo llvm-cov --version`; a cobertura roda em todo PR
(`ci / rust`), então um toolchain incompleto no CI falha na hora.
