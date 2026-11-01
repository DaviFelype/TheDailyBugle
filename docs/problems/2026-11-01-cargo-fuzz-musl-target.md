# 2026-11-01 — `cargo fuzz run` compila para musl e falha

- **Status:** resolvido
- **Commits/PRs:** `test(fuzz)` e `ci(dast)` (branch `ci/pipeline`)

## Sintoma

```
error[E0463]: can't find crate for `core`
Error: failed to build fuzz script: ... "--target" "x86_64-unknown-linux-musl" ...
```

## Causa raiz

Sem `--target`, o cargo-fuzz usa como padrão o alvo para o qual **ele próprio** foi
compilado. O mise instala o cargo-fuzz pelo cargo-binstall, que baixa o binário
pronto para musl. O nightly de fuzzing não tem a std de musl instalada.

## Solução

Passar `--target x86_64-unknown-linux-gnu` sempre. No `dast.yml` isso fica na
variável `FUZZ_TARGET_TRIPLE`.

## Como evitar

O `dast.yml` roda em todo PR. O CONTRIBUTING mostra o comando completo.
