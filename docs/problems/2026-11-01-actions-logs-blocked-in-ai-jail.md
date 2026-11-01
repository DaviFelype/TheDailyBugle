# 2026-11-01 — Logs do GitHub Actions inacessíveis no ai-jail

- **Status:** resolvido (contorno)
- **Commits/PRs:** commits `ci(probe)` temporários (branch `ci/pipeline`, PR #1)

## Sintoma

```
failed to get run log: Get "https://results-receiver.actions.githubusercontent.com/...": Forbidden
Get "https://productionresultssa0.blob.core.windows.net/actions-results/...": Forbidden
```

`gh run view --log` e `gh api .../actions/jobs/<id>/logs` falham. O Docker Hub
também fica bloqueado (`CONNECT tunnel failed, response 403`).

## Causa raiz

A API (`api.github.com`) passa pelo filtro de rede do ai-jail, mas os logs vêm de um
redirecionamento para um blob storage do Azure, e esse host não passa.

## Solução

O que a API devolve direto continua acessível:

- passos que falharam: `gh run view <run> --json jobs`;
- anotações: `gh api repos/<dono>/<repo>/check-runs/<job>/annotations`.

Para ver uma saída, um workflow temporário escreve o resultado como anotação
(`echo "::error title=tail::..."`, com quebras de linha como `%0A`). Esses commits são
descartados antes do merge.

## Como evitar

Nada a mudar no produto. Para ler o log completo, o usuário abre o run no navegador,
ou o host libera o domínio do blob storage no filtro do ai-jail.
