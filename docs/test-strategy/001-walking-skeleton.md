# 001 — Walking skeleton

- **Status:** aprovada em 2026-11-03
- **Fase do roadmap:** 2
- **PR:** <link quando existir>

## Objetivo

Ligar as duas pontas com dados de amostra: `thedailybugle feed --json` imprime o
contrato JSON v1, e o plugin mostra um ícone no bar e um popup com a lista. Nenhuma
rede ainda. O que esta leva fixa é o **contrato** e as regras de fronteira; as
fontes reais (Fase 3) só vão preencher esse contrato.

## Escopo

- **Dentro:**
  - CLI com `clap`: `thedailybugle feed --json` e `thedailybugle --version`.
  - Tipos do contrato que só aceitam valores válidos: `PlainText`, `HttpsUrl`,
    `ItemId`, `Category`, `Item`, `Feed`.
  - Itens de amostra embutidos no binário (`src/sample.rs`), um por categoria,
    marcados como amostra no título e na fonte.
  - JSON Schema do contrato em `schema/feed-v1.schema.json`.
  - Plugin: `BarWidget.qml` (ícone + contador), `Panel.qml` (popup com a lista,
    clique abre o link), `Model.js` (parse defensivo do JSON e regras de exibição).
  - Novos alvos de fuzzing (`plain_text`, `https_url`) no lugar de `parse_command`.
  - Regra Semgrep nova: QML não chama shell em `Process`.
  - Instalação local documentada no README: `cargo install --path . --locked` e
    `omarchy plugin add <caminho do repositório> --enable`.
- **Fora:**
  - Rede, cache, config TOML (Fases 3 e 4).
  - Limpeza de HTML (tags e entidades): depende do formato de cada fonte, entra na
    Fase 3. Aqui `<b>` aparece literal, o que é inofensivo com `Text.PlainText`.
  - Data de publicação, severidade, país: entram com as fontes que trazem esses
    dados. O contrato v1 aceita campos novos; o `Model.js` ignora campos que não
    conhece.
  - Abas, settings, i18n, notificações (Fase 5). Primeira release com tag (fica
    para depois do merge, num PR próprio de versão).

## Regras de negócio

1. `thedailybugle feed --json` imprime **uma** linha de JSON em stdout e sai com 0.
   Formato:
   ```json
   {"schemaVersion":1,"items":[{"id":"sample:vulnerability","category":"vulnerability",
     "title":"…","url":"https://…","source":"…"}]}
   ```
2. `category` é um de `vulnerability`, `ransomware`, `news`.
3. Argumentos errados: uso em stderr, nada em stdout, saída 2 (como hoje).
   `feed` sem `--json` também é erro de uso (não há saída para humanos ainda).
4. **`PlainText`** (título e fonte): remove caracteres de controle C0/C1 e de
   direção Unicode (U+200E, U+200F, U+061C, U+202A–U+202E, U+2066–U+2069);
   troca quebras de linha e tabs por espaço; junta espaços repetidos; tira espaços
   das pontas; corta no limite (título 300, fonte 64 caracteres) sem partir um
   caractere, terminando em `…`. Vazio depois da limpeza é erro.
5. **`HttpsUrl`**: interpretada pela crate `url`; esquema `https`; host presente;
   sem usuário/senha (`https://banco.com@mal.com`); no máximo 2048 bytes. Qualquer
   outra coisa é erro.
6. **`ItemId`**: 1–128 caracteres de `[A-Za-z0-9:._-]`.
7. Um `Feed` tem no máximo 200 itens. Item inválido não entra no feed (as fontes
   da Fase 3 vão registrar o descarte em stderr).
8. **Plugin.** O widget roda `thedailybugle feed --json` ao carregar, a cada 15 min
   e no clique do meio. O `Model.js` decide o estado:
   - `ok`: JSON válido com `schemaVersion` 1 → lista de itens.
   - `error`: binário ausente, saída diferente de 0, saída vazia ou JSON inválido.
   - `incompatible`: `schemaVersion` diferente de 1.

   Em `error`/`incompatible` o bar mostra um glifo de aviso e o popup, uma
   mensagem fixa (nunca o stderr do binário). Os últimos itens bons continuam
   visíveis.
9. **Defesa em profundidade no `Model.js`.** Mesmo confiando no binário, o modelo
   descarta itens sem `id`/`title`/`url` em string, com `url` que não começa com
   `https://` ou com categoria desconhecida; corta títulos em 300 caracteres e a
   lista em 200 itens.
10. **Links.** Clique num item roda `Process` com `["xdg-open", url]`, só se
    `Model.isOpenableUrl(url)` for verdadeiro. Sem shell, sem `Qt.openUrlExternally`.
11. Todo texto externo no QML usa `textFormat: Text.PlainText`.
12. Contador do bar: na Fase 2 é o número de itens (o "novo desde a última leitura"
    é da Fase 4), com o `barLabel` que já existe (`99+`).

## Casos de teste

| ID | Cenário | Entrada | Resultado esperado | Tipo |
|---|---|---|---|---|
| T01 | `feed --json` | processo real | stdout = uma linha de JSON, stderr vazio, saída 0 | integração |
| T02 | Saída segue o schema | stdout de T01 | válida contra `schema/feed-v1.schema.json` | integração |
| T03 | Saída estável | stdout de T01 | igual ao snapshot `insta` | snapshot |
| T04 | Amostra cobre as categorias | `sample::feed()` | um item de cada categoria | unidade |
| T05 | `--version` continua igual | processo real | como T06 da 000 | integração |
| T06 | Erros de uso | `feed`, `feed --xml`, `--foo`, argumento não UTF-8 | uso em stderr, stdout vazio, saída 2 | integração |
| T07 | Texto simples passa intacto | `"Ransomware hits X"` | igual | unidade |
| T08 | Espaços normalizados | `"  a\n\tb  "` | `"a b"` | unidade |
| T09 | Corte no limite | título com 301 caracteres, com emoji na borda | 300 caracteres terminando em `…`, UTF-8 válido | unidade |
| T10 | Vazio é erro | `""`, `"   "`, `"\u{202E}"` | `Err` | unidade |
| T11 | URL válida | `https://example.com/a?b=c` | `Ok` | unidade |
| T12 | `ItemId` | `sample:news` ok; `""`, `a b`, 129 caracteres, `../x` | ok; `Err` nos demais | unidade |
| T13 | Feed com mais de 200 itens | 201 itens | 200 itens | unidade |
| T14 | Schema rejeita saída ruim | JSON com `url` `http://`, sem `title`, `schemaVersion` 2 | inválido | unidade |
| T15 | `parseFeed` ok | JSON da amostra | estado `ok`, 3 itens | unidade (node) |
| T16 | `parseFeed` erro | saída ≠ 0, texto vazio, `"{"`, `"null"`, `"[]"` | estado `error`, sem exceção | unidade (node) |
| T17 | `parseFeed` incompatível | `schemaVersion` 2 | estado `incompatible` | unidade (node) |
| T18 | `parseFeed` ignora campos novos | item com `severity` extra | item aceito | unidade (node) |
| T19 | Estado de erro mantém itens | `ok` seguido de `error` | itens anteriores preservados | unidade (node) |
| T20 | Rótulo do bar em erro | estado `error` | glifo de aviso, não número | unidade (node) |
| T21 | QML compila | `BarWidget.qml`, `Panel.qml` | `qmllint` sem avisos | integração |
| T22 | Manifesto | `manifest.json` | validador sai com 0 | integração |
| M01 | Instalação local | `cargo install` + `omarchy plugin add <repo> --enable` | ícone no bar, popup com 3 itens | manual (você) |
| M02 | Binário ausente | renomear o binário e recarregar | glifo de aviso, bar continua de pé | manual (você) |
| M03 | Clique no item | clicar num item da amostra | abre no navegador padrão | manual (você) |

## Casos de segurança

| ID | Ataque ou falha | Entrada hostil | Resultado esperado |
|---|---|---|---|
| S01 | Bidi para disfarçar texto (Trojan Source) | `"abc\u{202E}gpj.exe"` | `"abcgpj.exe"` |
| S02 | Controle e escapes de terminal | `"\u{1b}[31mx\u{0}\u{85}"` | `"[31mx"` |
| S03 | Esquema perigoso | `javascript:`, `file:///etc/passwd`, `data:`, `http://`, `HTTPS://` | `Err` nos quatro primeiros; o último normalizado para `https://` e aceito |
| S04 | Credenciais na URL | `https://banco.com@mal.com/` | `Err` |
| S05 | URL gigante | 2049 bytes | `Err` |
| S06 | URL com espaço ou controle | `https://a.com/\nx`; `https://a .com` | primeira: a quebra é removida (regra WHATWG) e fica `https://a.com/x`; segunda: `Err` |
| S07 | Marcação QML/HTML no título chega ao QML | título `<img src=x>` | aparece literal; teste Semgrep já cobre `Text.PlainText` |
| S08 | JSON hostil no `Model.js` | `__proto__` como chave, `url: "javascript:alert(1)"`, `title: {}` | protótipo intacto; itens descartados |
| S09 | Lista gigante no `Model.js` | 10 000 itens | 200 itens |
| S10 | Link não `https` no clique | `isOpenableUrl("file:///x")`, `"http://x"`, `" https://x"` | `false` |
| S11 | Shell no QML | `Process { command: ["sh", "-c", …] }` | Semgrep bloqueia (regra nova, com arquivo de teste) |
| S12 | stderr do binário exibido | binário falha com mensagem qualquer | popup mostra só a mensagem fixa |
| S13 | Pânico em texto arbitrário | qualquer `&str` em `PlainText::new` | nunca entra em pânico; resultado sem controle/bidi e dentro do limite (fuzz) |
| S14 | Pânico em URL arbitrária | qualquer `&str` em `HttpsUrl::parse` | nunca entra em pânico; `Ok` só com `https` e sem credenciais (fuzz) |

## Dados de teste

- `tests/fixtures/feed-v1/` com JSONs válidos e inválidos para T14–T19 e S08
  (escritos à mão; sem fonte externa nesta fase).
- Amostra embutida em `src/sample.rs`, com URLs em `https://example.com/…`.
- Snapshot `insta` em `tests/snapshots/`.

### Dependências novas (pedido de aprovação)

| Crate | Versão | Publicada em | Uso | Por quê |
|---|---|---|---|---|
| `clap` | 4.6.7 | 2026-09-14 | binário | Prevista na seção 2 do roadmap. API builder, sem `derive`, para não trazer macros procedurais. |
| `serde` | 1.0.229 | 2026-07-18 | binário | Serialização do contrato (`derive`). |
| `serde_json` | 1.0.154 | 2026-10-11 | binário | Saída JSON. |
| `url` | 2.5.8 | 2026-01-05 | binário | Parser de URL padrão (WHATWG); o `reqwest` da Fase 3 já depende dela. |
| `jsonschema` | 0.60.2 | 2026-10-26 | só testes | Validar a saída contra o schema (T02, T14). Sem as features de rede e arquivo. |
| `insta` | 1.49.0 | 2026-10-03 | só testes | Snapshot do contrato (T03), previsto no roadmap. |

Todas passam por `cargo deny` e `cargo audit` no CI. O `scripts/sandbox.sh` continua
compilando sem rede (`cargo fetch` antes).

## Critério de pronto

- [ ] T01–T22 e S01–S14 automatizados e passando.
- [ ] M01–M03 conferidos por você na sua máquina.
- [ ] Cobertura de linhas ≥ 90%.
- [ ] CI verde (incluindo os dois alvos novos de fuzzing).
- [ ] README com instalação local e o contrato JSON.

## Riscos

- **PATH do omarchy-shell.** O `cargo install` põe o binário em `~/.cargo/bin`,
  que pode não estar no PATH do shell gráfico. Mitigação: o estado `error` mostra
  "thedailybugle não encontrado"; o README explica. O pacote AUR (Fase 6) resolve.
- **API interna do Quattro** (`Panel`, `KeyboardPanel`, `BarIconButton`) não é
  documentada e pode mudar. Mitigação: copiar o padrão do plugin de clima, que vem
  com o próprio Omarchy; `qmllint` no CI pega o que mudar de assinatura.
- **`jsonschema` traz várias crates** só para testes. Aceito por ser só
  `dev-dependency`; se o `cargo deny` reclamar, a alternativa é validar o schema no
  `node` com um validador mínimo escrito por nós.
- **Árvore do `clap`** é maior que o parser manual atual. Aceito por estar no
  roadmap; sem `derive` ela fica menor.
- **Teste visual do QML** fica manual (M01–M03). O teste de runtime do plugin é de
  uma fase posterior.
