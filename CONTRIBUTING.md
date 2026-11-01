# Como contribuir

## Ambiente

```sh
scripts/bootstrap.sh
```

Instala as versões fixadas em `mise.toml` e `rust-toolchain.toml` e ativa os hooks
do git (lefthook). Pode rodar de novo quando quiser; ele só instala o que falta.

## Fluxo de trabalho

1. **Estratégia de teste**: copie `docs/test-strategy/_template.md` para
   `docs/test-strategy/NNN-<tema>.md` e preencha. Ela precisa ser aprovada antes de
   qualquer código de produto.
2. **RED**: commit com os testes falhando (`test(escopo): ...`).
3. **GREEN**: commit com a implementação mínima (`feat(...)` ou `fix(...)`).
4. **Refatoração**: commits separados (`refactor(...)`).
5. **PR** para `main`. O merge só acontece com o CI verde e por **rebase**, para
   preservar os commits RED/GREEN.

## Branches

`feat/…`, `fix/…`, `chore/…`, `docs/…`, `test/…`, `ci/…`. Ninguém faz commit
direto em `main`.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/), verificados pelo
`cog verify` no hook `commit-msg`:

```
tipo(escopo opcional): descrição curta no imperativo, em inglês
```

Tipos: `feat`, `fix`, `test`, `refactor`, `docs`, `ci`, `chore`, `build`, `perf`,
`security`. Mudanças incompatíveis usam `!` (`feat!: ...`) ou o rodapé
`BREAKING CHANGE:`.

## Hooks locais

| Hook | Verificações |
|---|---|
| `pre-commit` | `gitleaks`, `cargo fmt --check`, `cargo clippy -D warnings`, `actionlint` e `zizmor` nos workflows |
| `commit-msg` | `cog verify` |

Não use `--no-verify`. Se um hook falhar sem motivo claro, registre em
`docs/problems/`.

## Dependências

Peça antes de adicionar uma dependência e justifique no PR. Versões fixas, e só
releases publicadas há pelo menos 7 dias.

## Registro de problemas

Problema que levou tempo para entender vira um arquivo em `docs/problems/`, a partir
de `docs/problems/_template.md`.
