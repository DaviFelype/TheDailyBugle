# 2026-11-01 — `semgrep --test` não acha os testes das regras

- **Status:** resolvido
- **Commits/PRs:** `security(semgrep)` (branch `ci/pipeline`)

## Sintoma

```
$ semgrep --test .semgrep/
No unit tests found.
```

As mesmas regras, copiadas para uma pasta sem ponto no nome: `4/4: ✓ All tests passed`.

## Causa raiz

O modo de teste do semgrep ignora pastas ocultas. Não tem relação com o
`.semgrepignore` (testado sem ele).

## Solução

Regras em `semgrep/` em vez de `.semgrep/`. O `.semgrepignore` exclui essa pasta do
scan normal, porque os arquivos de teste contêm os padrões de propósito.

## Como evitar

O job `security / semgrep` roda `semgrep --test semgrep/` antes do scan; se os testes
sumirem de novo, a saída "No unit tests found" aparece no log.
