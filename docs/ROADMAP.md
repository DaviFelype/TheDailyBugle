# Roadmap — TheDailyBugle

Widget para o Omarchy 4 (shell **Quattro**) com um feed rápido de cibersegurança:
vulnerabilidades novas com PoC, grandes ataques de ransomware, ransomware no Brasil
e notícias relevantes do dia a dia.

> Status: **aprovado em 2026-11-01** (decisões D1–D6 na seção 7).

---

## 1. Como o ai-usagebar funciona (e o que vamos replicar)

Estudei o repositório. Ele tem duas partes separadas:

```
┌─────────────────────────────┐      JSON (stdout)      ┌──────────────────────────────┐
│  Binário Rust (ai-usagebar) │ ──────────────────────► │  Plugin QML (Quattro)        │
│  - busca dados nas APIs     │   `ai-usagebar usage    │  - BarWidget.qml (ícone/bar) │
│  - cache em disco           │        --json`          │  - Panel.qml (popup)         │
│  - toda a lógica sensível   │                         │  - Model.js (lógica pura,    │
│  - testes: cargo test       │                         │    testada com node)         │
└─────────────────────────────┘                         └──────────────────────────────┘
        instalado via AUR                                instalado via `omarchy plugin add`
```

- `manifest.json` na raiz declara o plugin (`kinds: ["bar-widget"]`, entry point
  `omarchy/BarWidget.qml`, defaults e schema das configurações).
- O plugin só **exibe**. Rede, parsing e cache ficam no binário.
- A lógica do frontend fica em `Model.js` (JavaScript puro) para poder ser testada
  com `node` sem abrir o shell.
- `omarchy plugin validate .` valida o manifesto (já existe na sua máquina).

**Sim, dá para replicar.** E essa separação é a escolha certa para nós por um motivo
de segurança: todo conteúdo vindo da internet (títulos de notícias, nomes de vítimas,
descrições de CVE) é **entrada não confiável**. Ela é tratada, validada e
sanitizada no binário, com testes e fuzzing, antes de chegar ao QML.

## 2. Stack proposta

| Camada | Escolha | Por quê |
|---|---|---|
| Backend | **Rust** (edição 2024, toolchain estável via `mise`) | Mesma stack do ai-usagebar; memória segura; binário único fácil de empacotar no AUR; o melhor ecossistema de supply chain (`cargo-audit`, `cargo-deny`, `cargo-fuzz`). |
| HTTP | `reqwest` + `rustls` (sem OpenSSL) | TLS em Rust puro, timeouts e limite de tamanho de resposta. |
| Parsing | `serde_json`, `feed-rs` (RSS/Atom), `quick-xml` | Maduros e fuzzáveis. |
| CLI | `clap` | Padrão de mercado. |
| Config | TOML em `~/.config/thedailybugle/config.toml` | Igual ao ai-usagebar. |
| Frontend | QML (Quickshell / Quattro) + `Model.js` | Exigência do Omarchy 4. |
| Testes backend | `cargo nextest`, `wiremock` (servidor HTTP falso), `insta` (snapshots), `cargo-llvm-cov` (cobertura) | TDD rápido e determinístico, sem depender da internet. |
| Testes frontend | `node --test` sobre `Model.js`, `qmllint` | Já instalados (`node` 26, `/usr/lib/qt6/bin/qmllint`). |
| Commits | Conventional Commits verificados por **cocogitto** (`cog`) | Ferramenta Rust; também gera CHANGELOG e versão. |
| Hooks locais | **lefthook** | Roda fmt, clippy, gitleaks e checagem da mensagem antes do commit. |

**Alternativa considerada:** Python. Seria mais rápido para escrever, mas exige
runtime e dependências na máquina, é mais difícil de empacotar como binário único e
perde a paridade com o ai-usagebar. Go seria uma opção boa também, mas não traz
vantagem clara sobre Rust aqui. Recomendo Rust.

## 3. Fontes de dados (propostas)

| Categoria | Fonte | Observação |
|---|---|---|
| CVE explorada | CISA KEV (JSON) | Vulnerabilidades exploradas ativamente. |
| CVE detalhes | NVD API 2.0 | Score CVSS; limite de taxa (chave opcional). |
| Probabilidade de exploração | FIRST EPSS API | Ajuda a ordenar o que importa. |
| PoC público | `nomi-sec/PoC-in-GitHub`, Exploit-DB | Cruzamento CVE → PoC. |
| Ransomware global | ransomware.live API | Vítimas recentes, grupos. |
| Ransomware Brasil | ransomware.live (filtro país `BR`) + alertas do CTIR Gov | Recorte nacional. |
| Notícias | RSS: BleepingComputer, The Hacker News, The Record, Krebs, SecurityWeek, CISO Advisor (BR), CERT.br | Lista configurável. |

Antes de implementar cada fonte, verifico os termos de uso e o formato atual da API
e registro isso na estratégia de teste correspondente.

## 4. Segurança no CI/CD

### SAST e análise estática
| Ferramenta | O que cobre |
|---|---|
| `cargo clippy -D warnings` (+ lints `pedantic` selecionados) | Bugs e más práticas em Rust. |
| **Semgrep** (regras Rust, JS, secrets, GitHub Actions) | Padrões inseguros. |
| **CodeQL** (Rust e JavaScript) | Análise de fluxo de dados; resultado no GitHub Security. |
| `qmllint` + ESLint em `Model.js` | Erros no frontend. |

### SCA, segredos e supply chain
| Ferramenta | O que cobre |
|---|---|
| `cargo-audit` + `cargo-deny` | CVEs em dependências, licenças, crates proibidas, fontes não confiáveis. |
| OSV-Scanner | Segunda opinião sobre dependências. |
| **gitleaks** (CI e pre-commit) | Segredos commitados. |
| **zizmor** + **actionlint** | Segurança dos próprios workflows do GitHub Actions. |
| Dependabot | Atualização de crates e actions (actions fixadas por SHA). |
| OpenSSF Scorecard | Nota de maturidade de segurança do repositório. |
| SBOM (`cargo-cyclonedx`) + GitHub Artifact Attestations | Release verificável. |

### DAST — adaptado ao nosso caso
DAST clássico (OWASP ZAP etc.) ataca uma aplicação web em execução. Nosso plugin não
expõe servidor; a superfície de ataque é o **conteúdo hostil que chega das fontes
externas**. Então o DAST aqui roda o binário compilado contra entradas maliciosas:

1. **Fuzzing** (`cargo-fuzz`) dos parsers de JSON, RSS e XML. Sessão curta em todo
   PR, sessão longa agendada (noturna).
2. **Testes de feed hostil**: um servidor falso (`wiremock`) serve respostas
   maliciosas e o binário real é executado contra ele. Casos: XML bomb, resposta
   gigante, redirecionamento para `http://`/`file://`, timeout, JSON malformado,
   HTML/JS/marcação QML em títulos, links `javascript:`, caracteres de controle e
   Unicode bidi.
3. **Contrato de saída**: o JSON emitido para o QML é validado contra um JSON Schema;
   texto sempre sem marcação, URLs apenas `https://`.
4. **Teste de runtime do plugin** (fase posterior): carregar o QML com dados hostis
   e confirmar que nada é interpretado como rich text.

### Regras de segurança do produto (viram testes)
- O widget nunca quebra o bar: falha vira estado de erro, nunca crash.
- Só HTTPS, com timeout e limite de tamanho por resposta.
- Texto de fontes externas é exibido como `Text.PlainText`, nunca `RichText`.
- Links só abrem se forem `https://` e via `xdg-open`, sem shell.
- Cache e config com permissão `0600`/`0700`.
- Nenhuma telemetria.

## 5. Processo de desenvolvimento

### Ciclo de cada leva de trabalho
```
1. Estratégia de teste  →  docs/test-strategy/NNN-<tema>.md
2. Sua aprovação        →  (eu só continuo depois disso)
3. Testes falhando      →  commit `test(...)`        [RED]
4. Implementação mínima →  commit `feat(...)`/`fix(...)` [GREEN]
5. Refatoração          →  commit `refactor(...)`
6. PR com CI verde      →  merge em main (estratégia de merge: ver D5)
```

### Documento de estratégia de teste (modelo)
Cada documento terá, de forma resumida: objetivo, escopo (dentro/fora), regras de
negócio, tabela de casos de teste (ID, cenário, entrada, resultado esperado, tipo),
casos de segurança, dados de teste/fixtures, critério de pronto e riscos.

### Documentação
| Pasta/arquivo | Conteúdo |
|---|---|
| `README.md` | Instalação, uso, arquitetura com diagrama. |
| `CLAUDE.md` | Regras curtas e imperativas para o agente (modelo do artigo do Akita: estilo, comentários, testes, dependências, estrutura, formatação, logging, código defensivo). Menos de 100 linhas. |
| `CONTRIBUTING.md` | Conventional Commits, nomes de branch, fluxo de PR. |
| `SECURITY.md` | Como reportar vulnerabilidades. |
| `CHANGELOG.md` | Gerado pelo cocogitto a partir dos commits. |
| `docs/adr/` | Decisões de arquitetura (ADR), uma por arquivo. |
| `docs/test-strategy/` | Uma estratégia por leva de testes. |
| `docs/problems/` | Registro de problemas: sintoma, causa raiz, solução, como evitar, commits/PRs relacionados. |

### Git
- `main` protegida: só via PR, CI obrigatório, histórico linear.
- Branches: `feat/…`, `fix/…`, `chore/…`, `docs/…`, `test/…`, `ci/…`.
- Commits: `tipo(escopo): descrição` — tipos `feat`, `fix`, `test`, `refactor`,
  `docs`, `ci`, `chore`, `build`, `perf`, `security`.
- Versões SemVer derivadas dos commits; tags imutáveis.

---

## 6. Fases

### Fase 0 — Decisões e ambiente (sem código de produto)
- [x] **0.1 Decisões** (ver seção 7).
- [x] **0.2 Ferramentas locais** (instaladas no host via `scripts/bootstrap.sh`, conferidas em 2026-11-01): `gh auth login`; Rust via `mise`; `cargo-nextest`,
  `cargo-audit`, `cargo-deny`, `cargo-llvm-cov`, `cargo-fuzz` (nightly), `cargo-machete`,
  `cocogitto`; `lefthook`, `gitleaks`, `semgrep`, `actionlint`, `zizmor`.
  Script idempotente `scripts/bootstrap.sh` para reproduzir em máquina limpa.
- [x] **0.3 Repositório GitHub** (LICENSE, `.gitignore`, merge só por rebase, secret scanning, push protection, private vulnerability reporting, alertas do Dependabot; `main` protegida pelo ruleset `protect-main` em 2026-11-01. Checks obrigatórios entram ao fim da Fase 1. `dependabot.yml` fica para a Fase 1, quando existirem `Cargo.toml` e workflows): criação, LICENSE, `.gitignore`, proteção de `main`,
  secret scanning + push protection, private vulnerability reporting, Dependabot.
- [x] **0.4 Documentação base**: README, CLAUDE.md, CONTRIBUTING, SECURITY,
  ADR-0001 (stack), ADR-0002 (arquitetura binário + plugin), modelos de estratégia
  de teste e de registro de problema.
- [x] **0.5 Hooks locais** (instalados pelo bootstrap): lefthook com `cargo fmt`, `clippy`, `gitleaks` e
  `cog verify` na mensagem de commit.

**Pronto quando:** repositório público/privado criado, `main` protegida, um commit
`chore: initial project scaffold` passando nos hooks.

### Fase 1 — Pipeline CI/CD (antes de qualquer funcionalidade)
- [ ] `ci.yml`: fmt → clippy → testes + cobertura → `qmllint` + testes `Model.js`
  → `omarchy plugin validate` (equivalente em CI) → checagem de commits.
- [ ] `security.yml`: Semgrep, CodeQL, cargo-audit/deny, OSV-Scanner, gitleaks,
  zizmor, actionlint, Scorecard.
- [ ] `dast.yml`: fuzzing curto em PR, longo agendado; testes de feed hostil.
- [ ] `release.yml`: tag → build x86_64/aarch64 → SBOM → attestation → GitHub Release.
- [ ] Projeto "hello world" mínimo só para o pipeline ter o que verificar.

**Pronto quando:** PR de teste com uma vulnerabilidade proposital é bloqueado pelo CI
(provando que os gates funcionam), e documentado em `docs/problems/` se algo falhar.

### Fase 2 — Walking skeleton (ponta a ponta com dados falsos)
- [ ] Estratégia de teste 001.
- [ ] `thedailybugle feed --json` devolve itens de fixture conforme o JSON Schema.
- [ ] Plugin mínimo: ícone no bar + popup listando os itens.
- [ ] Instalação local via `omarchy plugin add` apontando para o repositório.

### Fase 3 — Fontes de dados (uma estratégia de teste por fonte)
- [ ] 3.1 CVEs com PoC (KEV + NVD + EPSS + PoC-in-GitHub).
- [ ] 3.2 Ransomware global.
- [ ] 3.3 Ransomware no Brasil.
- [ ] 3.4 Notícias (RSS configurável).

### Fase 4 — Agregação
- [ ] Cache em disco com TTL por fonte (respeitar limites das APIs).
- [ ] Deduplicação, ordenação por relevância (KEV, EPSS, CVSS, recência, Brasil).
- [ ] Marcação de "novo desde a última leitura".
- [ ] Configuração TOML com validação.

### Fase 5 — Frontend completo
- [ ] Bar: ícone + contador de itens críticos novos.
- [ ] Painel com abas: CVEs/PoC · Ransomware · Brasil · Notícias.
- [ ] Settings no próprio painel; i18n pt-BR e en; tema do Omarchy.
- [ ] Notificação opcional para itens críticos (ex.: CVE no KEV com PoC).

### Fase 6 — Distribuição
- [ ] Pacote AUR (`thedailybugle-bin`) publicado pelo CI.
- [ ] Publicação em plugins.omarchy.org.
- [ ] Release 1.0.0.

---

## 7. Decisões (aprovadas em 2026-11-01)

| # | Decisão | Resultado |
|---|---|---|
| D1 | Stack do backend | Rust (stack da seção 2) |
| D2 | Nome do projeto | **TheDailyBugle** — ID do plugin e nome do binário: ver D7 |
| D3 | Repositório | Público: https://github.com/DaviFelype/TheDailyBugle |
| D4 | Licença | MIT |
| D5 | Estratégia de merge | Rebase/merge linear, preservando os commits RED/GREEN do TDD |
| D6 | Idioma | Código, commits e CLAUDE.md em inglês; documentação de processo em pt-BR |
| D7 | ID do plugin / binário / pasta de config | `davifelype.thedailybugle` / `thedailybugle` / `~/.config/thedailybugle/` |
| D8 | Autor dos commits | Davi Felype <davifelypecostasilva@gmail.com> (e-mail pessoal, público no histórico) |
