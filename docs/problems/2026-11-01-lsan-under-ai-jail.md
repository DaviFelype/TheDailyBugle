# 2026-11-01 — Fuzzing "falha" no ai-jail com LeakSanitizer

- **Status:** resolvido (contorno só local)
- **Commits/PRs:** `test(fuzz)` (branch `ci/pipeline`)

## Sintoma

Depois de centenas de milhares de execuções sem erro:

```
==4834==LeakSanitizer has encountered a fatal error.
==4834==HINT: LeakSanitizer does not work under ptrace (strace, gdb, etc)
Test unit written to fuzz/artifacts/parse_command/crash-da39a3ee5e6b4b0d3255bfef95601890afd80709
```

O "crash" é a entrada vazia (`da39a3…` é o SHA-1 de zero bytes).

## Causa raiz

O LeakSanitizer roda ao encerrar e precisa de `ptrace`, que o seccomp do ai-jail
bloqueia. Não é um bug do código. `-detect_leaks=0` do libFuzzer não basta: a
checagem do ASan ao sair continua ligada.

## Solução

Dentro do ai-jail, rodar com `ASAN_OPTIONS=detect_leaks=0`. No CI e no host a
detecção de vazamento continua ligada.

## Como evitar

Documentado no CONTRIBUTING, na seção de fuzzing.
