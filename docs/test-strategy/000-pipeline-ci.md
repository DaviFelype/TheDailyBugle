# 000 — Pipeline de CI/CD

- **Status:** aprovada em 2026-11-01
- **Fase do roadmap:** 1
- **PR:** #1

## Objetivo

Colocar no ar os quatro workflows (`ci`, `security`, `dast`, `release`) e um
"hello world" mínimo que dê a eles algo para verificar. A leva só termina quando
PRs com falhas propositais forem bloqueados pelo CI.

## Escopo

- **Dentro:**
  - Crate `thedailybugle` (sem dependências) com `--version` e erro de uso.
  - Esqueleto do plugin: `manifest.json`, `omarchy/BarWidget.qml` estático,
    `omarchy/Model.js` com uma função pura.
  - Workflows `ci.yml`, `security.yml`, `dast.yml`, `release.yml`, `scorecard.yml`.
  - `deny.toml`, `dependabot.yml`, regras próprias do Semgrep em `.semgrep/`.
  - Cópia fixada do `omarchy-plugin-validate` para rodar no CI.
  - Checks obrigatórios no ruleset da `main`.
  - Isolamento da máquina de desenvolvimento (ver "Isolamento local"):
    `mise.ci.toml`, `scripts/sandbox.sh`, `rust-analyzer.toml`.
- **Fora:**
  - `feed --json`, JSON Schema, `clap`, rede (Fase 2).
  - Testes de feed hostil com `wiremock` (Fase 3: não há fonte ainda).
  - Teste de runtime do QML (fase posterior).
  - Primeira release com tag (Fase 2). Aqui o `release.yml` só roda em modo de ensaio.

## Regras de negócio

1. `thedailybugle --version` imprime `thedailybugle <versão do Cargo.toml>` em stdout
   e sai com 0.
2. Sem argumentos ou com argumento desconhecido: mensagem de uso em stderr, nada em
   stdout, código de saída 2.
3. O parser de argumentos é uma função pura (`parse_command`) que nunca entra em
   pânico. Ele existe só até a Fase 2, quando o `clap` entra; o alvo de fuzzing passa
   então para o contrato JSON.
4. `Model.js` exporta uma função pura de exibição (ex.: rótulo do bar a partir de um
   contador) e é carregado no `node` via `node:vm`, sem pacotes npm.
5. O manifesto passa no `omarchy plugin validate` local e na cópia usada no CI.

### Regras do pipeline

1. Toda action fixada por SHA de commit, com a versão em comentário. Releases com
   pelo menos 7 dias.
2. `permissions: {}` no topo de cada workflow; cada job pede só o que usa.
   `actions/checkout` com `persist-credentials: false`. Sem `pull_request_target`.
3. Ferramentas instaladas pelo `jdx/mise-action` a partir do `mise.toml` (as mesmas
   versões da máquina local) e do `mise.ci.toml` (`MISE_ENV=ci`, só no CI).
   Nada de `curl | sh`.
4. Todo job tem `timeout-minutes`. Workflows com `concurrency` para cancelar
   execuções antigas do mesmo PR.
5. Sem cache no `release.yml` (evita envenenamento de cache no artefato publicado).
6. Dependabot para `cargo` e `github-actions`, semanal, com `cooldown` de 7 dias
   (mesma regra das dependências manuais).

## Isolamento local

O risco na máquina de desenvolvimento não são as ferramentas fixadas, e sim o código
de terceiros que roda sem ninguém ver: `build.rs` e macros das crates (em todo
`cargo build`/`test`/`clippy`, inclusive no hook de pre-commit), alvos de fuzzing e o
rust-analyzer do editor. Docker não entra: o grupo `docker` equivale a root.

1. **Ferramentas só do CI ficam fora do host.** `osv-scanner`, `cargo-cyclonedx` e
   `semgrep` vão para `mise.ci.toml`; o CodeQL vem da própria action. O host fica só
   com o que o ciclo TDD e os hooks usam. O semgrep já instalado sai com
   `mise uninstall semgrep@1.182.0`.
2. **`scripts/sandbox.sh <comando>`** roda o comando com bubblewrap:
   - Rede: `cargo fetch` roda antes, fora do sandbox (só baixa, não executa nada);
     dentro, `--unshare-all` e cargo `--offline`.
   - Arquivos: `/` só leitura; `$HOME`, `/run` e `/tmp` viram tmpfs vazios (some
     `~/.ssh`, `~/.config/gh`, navegador e os sockets de D-Bus, Wayland e
     ssh-agent); voltam só leitura o toolchain do rustup, as instalações do mise e o
     registro do cargo (sem `credentials.toml`). O projeto fica **só leitura**,
     exceto `target/` e as pastas de saída do fuzz: assim nada planta hooks em
     `.git` nem altera `lefthook.yml` ou scripts que depois rodam no host.
     *(Endurecido durante a implementação; antes era "projeto com escrita, menos
     `.git`".)*
   - Ambiente: `--clearenv` com lista fixa (`PATH`, `HOME`, `TERM`, variáveis do
     cargo/rustup). Tokens como `GH_TOKEN` não entram.
   - `--die-with-parent` e `--new-session` (bloqueia injeção no terminal via
     `TIOCSTI`).
   - Já dentro de um namespace de usuário (ex.: ai-jail, onde bwrap aninhado falha
     com "Operation not permitted"): roda o comando direto e avisa em stderr. A
     detecção lê `/proc/self/uid_map`, não uma variável que um processo possa forjar.
3. **Hooks e fuzzing passam pelo sandbox.** O `clippy` do lefthook e o
   `cargo fuzz run` local chamam `scripts/sandbox.sh`. O CONTRIBUTING documenta
   `scripts/sandbox.sh cargo nextest run` como forma padrão de rodar os testes.
4. **`rust-analyzer.toml`** na raiz desliga a execução de `build.rs` e de macros
   procedurais pelo editor (`cargo.buildScripts.enable = false`,
   `procMacro.enable = false`).

## Workflows

| Workflow | Gatilho | Jobs |
|---|---|---|
| `ci.yml` | PR, push em `main` | `commits` (`cog check` no intervalo do PR) · `rust` (fmt, clippy `-D warnings`, `nextest`, `llvm-cov --fail-under-lines 90`) · `frontend` (`qmllint`, `node --test`, validação do manifesto) |
| `security.yml` | PR, push em `main`, semanal | `gitleaks` (histórico inteiro) · `cargo-audit` · `cargo-deny` · `osv-scanner` · `semgrep` (`p/rust`, `p/javascript`, `p/secrets`, `p/github-actions` + `semgrep/`, SARIF no GitHub) · `workflows` (`actionlint`, `zizmor`) · `codeql` (Rust, JavaScript, Actions) |
| `scorecard.yml` | push em `main`, semanal | OpenSSF Scorecard, publica resultado |
| `dast.yml` | PR (60 s por alvo), noturno (30 min por alvo) | `cargo fuzz run` em cada alvo; corpus e crashes como artefato |
| `release.yml` | tag `v*`, `workflow_dispatch` (ensaio) | build `x86_64` (`ubuntu-24.04`) e `aarch64` (`ubuntu-24.04-arm`) com `--locked` · tar.gz + SHA-256 · SBOM CycloneDX · attestation · `gh release create` (pulado no ensaio) |

Regras próprias do Semgrep (`semgrep/`, cada uma com arquivo de teste), tiradas das regras de segurança do produto:

- QML com `Text.RichText`, `Text.StyledText`, `Text.MarkdownText` ou `Text.AutoText`.
- QML com `Qt.openUrlExternally` (links só via `xdg-open` no binário/processo).
- Rust com `Command::new("sh" | "bash")` ou `-c`.

Lints do Cargo (`[lints]` no `Cargo.toml`): `unsafe_code = "forbid"`,
`clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic` como `deny`
(liberados em testes).

## Casos de teste

| ID | Cenário | Entrada | Resultado esperado | Tipo |
|---|---|---|---|---|
| T01 | Versão | `parse_command(["--version"])` | `Command::Version` | unidade |
| T02 | Sem argumentos | `parse_command([])` | `Err(UsageError)` | unidade |
| T03 | Argumento desconhecido | `parse_command(["--foo"])` | `Err(UsageError)` | unidade |
| T04 | Argumento extra | `parse_command(["--version", "x"])` | `Err(UsageError)` | unidade |
| T05 | Linha de versão | `version_line()` | `thedailybugle 0.1.0` (de `CARGO_PKG_VERSION`) | unidade |
| T06 | Binário: `--version` | processo real | stdout com a linha, stderr vazio, saída 0 | integração |
| T07 | Binário: uso errado | processo real com `--foo` | stdout vazio, uso em stderr, saída 2 | integração |
| T08 | Parser nunca entra em pânico | qualquer lista de strings | retorna `Ok` ou `Err` | propriedade (fuzz) |
| T09 | Rótulo do bar | `barLabel(0)`, `barLabel(3)`, `barLabel(150)` | `""`, `"3"`, `"99+"` | unidade (node) |
| T10 | Rótulo com entrada inválida | `barLabel(-1)`, `barLabel(NaN)`, `barLabel("3")` | `""` | unidade (node) |
| T11 | Manifesto válido | `manifest.json` | validador sai com 0 | integração |
| T12 | QML sem erros | `omarchy/*.qml` | `qmllint` sem avisos | estático |
| T13 | Testes rodam no sandbox | `scripts/sandbox.sh cargo nextest run` | testes passam, sem rede | integração (bash) |
| T14 | Código de saída preservado | `scripts/sandbox.sh sh -c 'exit 7'` | saída 7 | integração (bash) |
| T15 | Já dentro de namespace | `sandbox.sh` num namespace de usuário | roda direto, aviso em stderr, mesma saída | integração (bash) |
| T16 | Saída do build tem escrita | criar arquivo em `target/` | funciona | integração (bash) |

Os testes do sandbox (T13–T16, S13–S19) ficam em `tests/sandbox/run.sh`, em bash
puro. No CI, o job habilita namespaces de usuário no runner
(`kernel.apparmor_restrict_unprivileged_userns=0`) e instala o `bubblewrap` via apt.

## Casos de segurança (prova dos gates)

Cada caso é um PR descartável (`test/ci-gate-NN`), em rascunho. Esperado: CI vermelho
no job indicado. Depois o PR é fechado sem merge e a branch apagada. O resultado
(link do run) fica registrado na seção "Resultado" deste documento.

| ID | Falha proposital | Entrada | Job que deve bloquear |
|---|---|---|---|
| S01 | Aviso do clippy | variável não usada | `ci / rust` |
| S02 | Teste falhando | `assert_eq!(1, 2)` | `ci / rust` |
| S03 | Cobertura baixa | função nova sem teste | `ci / rust` (llvm-cov) |
| S04 | Commit fora do padrão | mensagem `update stuff` | `ci / commits` |
| S05 | Crate com CVE conhecida | versão antiga com aviso RUSTSEC no `Cargo.toml` | `cargo-audit`, `cargo-deny`, `osv-scanner` |
| S06 | Licença proibida | crate GPL | `cargo-deny` |
| S07 | Workflow inseguro | action por tag (`@v4`) + `pull_request_target` | `workflows` (zizmor) |
| S08 | Rich text no QML | `textFormat: Text.RichText` | `semgrep` (regra própria) |
| S09 | Shell no Rust | `Command::new("sh").arg("-c")` | `semgrep` (regra própria) |
| S10 | Segredo falso | chave com formato real, gerada na hora | push protection do GitHub ou `gitleaks` |
| S11 | `unwrap()` em código de produto | `"1".parse::<u8>().unwrap()` | `ci / rust` (clippy) |
| S12 | Manifesto inválido | `entryPoints.barWidget` apontando para arquivo inexistente | `ci / frontend` |

Casos do sandbox (automatizados em `tests/sandbox/run.sh`, não são PRs):

| ID | Ataque | Comando dentro do sandbox | Resultado esperado |
|---|---|---|---|
| S13 | Plantar hook do git | escrever em `.git/hooks/pre-commit` | falha (só leitura) |
| S14 | Exfiltrar por rede | conectar em `1.1.1.1:443` e resolver DNS | falha |
| S15 | Ler credenciais | ler `$HOME/.ssh`, `$HOME/.config/gh` | não existem |
| S16 | Ler token do ambiente | `GH_TOKEN=falso` fora; `echo $GH_TOKEN` dentro | vazio |
| S17 | Escrever fora do projeto | criar arquivo no `$HOME` e em `/tmp` | não persiste no host |
| S18 | Alterar arquivo que roda no host | criar arquivo na raiz do projeto | falha (só leitura) |
| S19 | Usar sockets do host | listar `/run` (D-Bus, Wayland, ssh-agent) | vazio |

Os hooks locais bloqueariam vários desses commits. Para não usar `--no-verify`, os
commits dos PRs S01–S12 são criados direto no GitHub pela API (`gh api`), como faria
um contribuidor sem os hooks. É exatamente o cenário que o CI precisa cobrir.

## Dados de teste

Sem fixtures nesta leva. O validador do Omarchy é copiado do upstream
(`basecamp/omarchy`, commit `b7e900c` de 2026-10-15, MIT) para `scripts/vendor/`,
com cabeçalho de origem e licença. É um pouco mais novo que o instalado no host
(4.0.0.alpha); o teste roda os dois quando o host tem Omarchy. O job `qml` usa o
`shell/` do mesmo commit para resolver os tipos `qs.Ui`.

### Versões a fixar (releases com 7+ dias em 2026-11-01)

| Item | Versão | SHA / data |
|---|---|---|
| `actions/checkout` | v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| `jdx/mise-action` | v5.2.3 | `82a612c5ab8bcda7cc5db6b7e81a8e7584c3dc6e` |
| `Swatinem/rust-cache` | v2.9.3 | `d1527c8f67ef9af357290475d4af4b0fab8f12e0` |
| `github/codeql-action` | v4.39.1 | `9b3254085494b55a0f7c4c2b3d2d52cc712ce370` |
| `ossf/scorecard-action` | v2.5.0 | `fa7117db83c52d787ae5a89f350999701505d78e` |
| `actions/upload-artifact` | v7.0.2 | `cf430e030ddbb5b0abf93d22962f4752f3646cd9` |
| `actions/download-artifact` | v8.0.2 | `9000827ccba6bdab643e8b6fd33ac0654aef8333` |
| `actions/attest-build-provenance` | v4.2.2 | `4d101475d8b20a2381f78447822ac1eab6504dd8` |
| mise (instalado pelo `jdx/mise-action`) | 2026.10.19 | 2026-10-24 |
| `osv-scanner` (`mise.ci.toml`, aqua) | 2.7.0 | 2026-10-12 |
| `cargo-cyclonedx` (`mise.ci.toml`, cargo) | 0.5.9 | 2026-03-19 |
| `semgrep` (movido para `mise.ci.toml`) | 1.182.0 | já fixado |
| `bubblewrap` | o do sistema (Arch no host, apt no CI) | — |
| `node` (mise) | 26.12.0 | já instalado no host |
| `libfuzzer-sys` (só em `fuzz/`, não vai no binário) | 0.4.13 | 2026-06-04 |

## Critério de pronto

- [x] T01–T16 e S13–S19 automatizados e passando.
- [x] Cobertura de linhas ≥ 90%.
- [x] Todos os workflows verdes no PR desta leva; `release.yml` verde em ensaio.
- [x] S01–S12 vermelhos no job esperado, com links registrados aqui.
- [ ] Ruleset da `main` exige os jobs de `ci` e `security`.
- [x] Falhas não óbvias registradas em `docs/problems/`.

## Riscos

- **`qmllint` diferente do local.** O Ubuntu 24.04 traz Qt 6.4; o host tem 6.12.
  Mitigação: job `frontend` num container `archlinux` fixado por digest.
- **CodeQL para Rust** ainda é recente e pode gerar ruído. Mitigação: começa como
  check não obrigatório; vira obrigatório depois de uma semana sem falso positivo.
- **Semgrep `p/*`** baixa regras na hora; o conjunto muda sem aviso. Mitigação: só
  as regras próprias de `.semgrep/` bloqueiam; as do registro vão para o painel de
  segurança.
- **Fuzzing de um parser trivial** prova o pipeline, não o produto. Aceito: o alvo
  troca na Fase 2.
- **Sandbox pulado dentro de outro namespace.** Dentro do ai-jail o próprio jail
  já isola; em outros namespaces (flatpak, toolbox) o isolamento depende deles.
  Aceito e documentado; o aviso em stderr deixa isso visível.
- **`rust-analyzer.toml`** é recente e alguns editores podem ignorar essas opções.
  Mitigação: o CONTRIBUTING mostra as mesmas opções para configurar no editor.
- **`sandbox.sh` incompleto** (algo do host continua visível). Mitigação: S13–S17
  rodam no CI e localmente; uma brecha nova vira caso de teste.
- **Scorecard** penaliza coisas fora do nosso controle (ex.: poucos mantenedores).
  Só informativo.

## Resultado

Registrado em 2026-11-01. Os PRs dos casos S01–S12 tinham como base a `ci/pipeline`
e foram criados pela API, sem hooks locais.

**PR desta leva (#1), todos os workflows verdes:** [ci](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351215011),
[security](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351214929), [dast](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351214985),
[release](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351215082) (ensaio: `publish` pulado fora de tag).

| ID | PR | Bloqueado por | Run |
|---|---|---|---|
| S01 | #2 | `ci / rust` (clippy) | [ci](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351795028) |
| S02 | #3 | `ci / rust` (testes); também `ci / sandbox` | [ci](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351815117) |
| S03 | #4 | `ci / rust` (llvm-cov abaixo de 90%) | [ci](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351821726) |
| S04 | #5 | `ci / commits` | [ci](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351827837) |
| S05 | #6 | `cargo-audit`, `cargo-deny`, `osv-scanner` (`smallvec 1.6.0`, RUSTSEC-2021-0003) | [security](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351834351) |
| S06 | #7 | `cargo-deny` (`lzo1x 0.2.2`, GPL-2.0) | [security](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351841181) |
| S07 | #8 | `security / workflows` | [security](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351848400) |
| S08 | #9 | `security / semgrep` (regra própria) | [security](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351857713) |
| S09 | #10 | `security / semgrep` (regra própria) | [security](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351862660) |
| S10 | — | push protection do GitHub: a API recusou o commit (`HTTP 422, Secret detected in content`), então nem branch nem PR chegaram a existir | — |
| S11 | #11 | `ci / rust` (clippy `unwrap_used`) | [ci](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351869144) |
| S12 | #12 | `ci / model-js` (passo "Plugin manifest") | [ci](https://github.com/DaviFelype/TheDailyBugle/actions/runs/41351874869) |

Observações:

- O job chamado `frontend` neste documento foi dividido em `model-js` (testes do
  `Model.js` e manifesto) e `qml` (`qmllint` no container Arch).
- Em S05 e S06 o `ci / sandbox` também falha: o sandbox compila sem rede, então
  uma crate nova fora do cache não baixa. Isso é esperado.
- Em S09 o `ci / rust` também falhou no `rustfmt`, porque a linha de teste passou
  de 100 colunas. Isso não muda o resultado: quem tinha que bloquear era o Semgrep, e
  ele bloqueou.
