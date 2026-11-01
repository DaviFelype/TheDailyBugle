# 2026-11-01 — Ferramentas `cargo:` não instalam no CI

- **Status:** resolvido
- **Commits/PRs:** `ci(setup-tools)` (branch `ci/pipeline`, PR #1)

## Sintoma

Todo job que instala uma ferramenta `cargo:` (`rust`, `sandbox`, `fuzz`,
`cargo-audit`, `release`) falha no passo `setup-tools`:

```
Failed to install cargo:cargo-nextest@0.9.149: tool 'cargo:cargo-nextest@0.9.149'
requires configured install dependency 'rust@1.99.1', but its selected version is
not installed
```

Os jobs que instalam só `cocogitto`, `node` etc. passam.

## Causa raiz

O `mise.toml` fixa `rust = "1.99.1"`. Com o `rust` na configuração, o mise
(2026.10.19) trata ele como dependência obrigatória do backend `cargo:` e se recusa a
instalar a ferramenta enquanto o `rust` não estiver instalado *pelo mise*. No host o
`bootstrap.sh` instala tudo, então o erro não aparece. No CI o toolchain vem do
`rustup toolchain install`, e o mise não conta esse toolchain como instalado.

## Solução

A action `setup-tools` roda `mise install rust` antes da lista quando alguma
ferramenta começa com `cargo:`. O mise usa o `rustup` e aproveita o toolchain que já
está lá.

## Como evitar

Todos os jobs do PR passam por essa action. Se um job novo instalar ferramenta
`cargo:` sem passar por ela, ele falha logo no CI.
