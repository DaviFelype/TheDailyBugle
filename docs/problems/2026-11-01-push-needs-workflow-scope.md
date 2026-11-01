# 2026-11-01 — Push recusado ao adicionar workflows

- **Status:** aberto (precisa de ação no host)
- **Commits/PRs:** branch `ci/pipeline`

## Sintoma

```
! [remote rejected] ci/pipeline -> ci/pipeline (refusing to allow an OAuth App to
create or update workflow `.github/workflows/ci.yml` without `workflow` scope)
```

## Causa raiz

O token OAuth do `gh` (usado pelo `gh auth git-credential` dentro do ai-jail) foi
criado sem o escopo `workflow`. O GitHub exige esse escopo para qualquer push que
crie ou altere arquivos em `.github/workflows/`.

## Solução

No host: `gh auth refresh -h github.com -s workflow`, e reiniciar a sessão do ai-jail
para o novo token entrar no ambiente.

## Como evitar

`scripts/bootstrap.sh` confere os escopos com `gh auth status` e avisa se faltar
`workflow`.
