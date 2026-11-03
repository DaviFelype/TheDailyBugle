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
| `pre-commit` | `gitleaks`, `cargo fmt --check`, `cargo clippy -D warnings` (dentro do sandbox), `actionlint` e `zizmor` nos workflows |
| `commit-msg` | `cog verify` |

Não use `--no-verify`. Se um hook falhar sem motivo claro, registre em
`docs/problems/`.

## Isolamento da máquina

Código de terceiros roda sem ninguém ver em todo build: `build.rs` e macros das
crates, alvos de fuzzing. Por isso:

- **Comandos que compilam ou executam código de crates passam pelo sandbox**
  (bubblewrap: sem rede, `HOME` vazio, projeto só leitura exceto `target/` e as
  pastas de saída do fuzz):

  ```sh
  scripts/sandbox.sh cargo nextest run
  scripts/sandbox.sh cargo clippy --all-targets -- -D warnings
  ```

  `cargo fmt` não executa código de crates e pode rodar direto. Dentro do ai-jail o
  sandbox não aninha; o script avisa e roda direto, e o isolamento fica com o jail.
- **Editor**: o `rust-analyzer.toml` desliga `build.rs` e macros procedurais. Se o
  seu editor ignorar o arquivo, configure `rust-analyzer.cargo.buildScripts.enable`
  e `rust-analyzer.procMacro.enable` como `false`.
- **Ferramentas só do CI** (`semgrep`, `osv-scanner`, `cargo-cyclonedx`) ficam em
  `mise.ci.toml` e não são instaladas pelo bootstrap.

## Testes

```sh
scripts/sandbox.sh cargo nextest run            # Rust
scripts/sandbox.sh cargo llvm-cov nextest       # Rust com cobertura (mínimo 90%)
node --test tests/js/                           # Model.js
tests/plugin/validate-manifest.sh               # manifesto do plugin
tests/plugin/qmllint.sh                         # QML contra os tipos do Omarchy
tests/sandbox/run.sh                            # o próprio sandbox
```

Fuzzing (o `--target` é obrigatório; ver `docs/problems/2026-11-01-cargo-fuzz-musl-target.md`):

```sh
scripts/sandbox.sh cargo +nightly-2026-10-15 fuzz run --target x86_64-unknown-linux-gnu plain_text -- -max_total_time=60
```

Dentro do ai-jail, acrescente `ASAN_OPTIONS=detect_leaks=0` (ver
`docs/problems/2026-11-01-lsan-under-ai-jail.md`).

## Dependências

Peça antes de adicionar uma dependência e justifique no PR. Versões fixas, e só
releases publicadas há pelo menos 7 dias.

## Registro de problemas

Problema que levou tempo para entender vira um arquivo em `docs/problems/`, a partir
de `docs/problems/_template.md`.
