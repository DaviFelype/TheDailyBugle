# 002 — CVEs exploradas com PoC (KEV + PoC-in-GitHub) e cache

- **Status:** aprovada em 2026-11-03
- **Fase do roadmap:** 3.1a (D9, D10)
- **PR:** <link quando existir>

## Objetivo

Trocar o item de amostra de vulnerabilidade por dados reais: as CVEs que a CISA
adicionou ao KEV nos últimos 30 dias, cada uma com o número de PoCs públicos no
GitHub. Junto entra a camada de rede (HTTPS, timeouts, limite de tamanho) e o cache
em disco com TTL por fonte, que as próximas fontes vão reusar.

## Escopo

- **Dentro:**
  - Cliente HTTP único (`reqwest` + rustls com `ring`), usado por todas as fontes.
  - Fonte KEV: `https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json`.
  - Fonte PoC: `https://raw.githubusercontent.com/nomi-sec/PoC-in-GitHub/master/<ano>/<CVE>.json`,
    uma requisição por CVE da janela.
  - Cache em `$XDG_CACHE_HOME/thedailybugle/` (padrão `~/.cache/thedailybugle/`).
  - Contrato v1 com campos novos opcionais (o v1 aceita adições; ver regra 9).
  - Plugin: segunda linha do item com data, contagem de PoC e ransomware; aviso de
    fonte que falhou.
  - Alvos de fuzzing para os parsers de KEV, PoC e cache.
- **Fora:**
  - CVSS (NVD) e EPSS: estratégia 003.
  - Ordenação por relevância, deduplicação entre fontes, "novo desde a última
    leitura", configuração TOML (Fase 4). A janela de 30 dias e os TTLs são
    constantes por enquanto.
  - Link para os repositórios de PoC (ver regra 6).
  - Os itens de amostra de ransomware e notícias continuam até as Fases 3.2–3.4.

## Regras de negócio

1. **Janela.** Entram as entradas do KEV com `dateAdded` nos últimos 30 dias
   (contando hoje, em UTC), da mais recente para a mais antiga; no máximo 100.
2. **Item de vulnerabilidade.**
   - `id`: `kev:<CVE>`; `category`: `vulnerability`; `source`: `CISA KEV`.
   - `title`: `<CVE> <vulnerabilityName>` (ex.: `CVE-2026-119977 WordPress WordPress
     Core Time-of-check Time-of-use (TOCTOU) Race Condition Vulnerability`), limpo
     como `PlainText`.
   - `url`: `https://nvd.nist.gov/vuln/detail/<CVE>`, montada por nós a partir do
     CVE validado, nunca copiada do feed.
3. **Validação do KEV** (entrada hostil). Uma entrada é descartada, com contagem em
   stderr, se:
   - `cveID` não casar com `^CVE-[0-9]{4}-[0-9]{4,19}$`;
   - `dateAdded` não for uma data real `AAAA-MM-DD` ou estiver no futuro;
   - `vulnerabilityName` ficar vazio depois da limpeza.

   CVEs repetidos: fica a primeira ocorrência. O feed inteiro é recusado se passar
   de 8 MiB, não for JSON, ou não tiver o array `vulnerabilities` (no máximo 20 000
   entradas).
4. **PoC.** Para cada CVE da janela, o arquivo do PoC-in-GitHub:
   - HTTP 200 com um array: `pocCount` = número de elementos que são objetos com
     `html_url` em `https://github.com/<dono>/<repo>` (no máximo 10 000).
   - HTTP 404: `pocCount` = 0 (o projeto ainda não tem PoC para o CVE).
   - Qualquer outra resposta, corpo acima de 4 MiB ou JSON inválido: `pocCount`
     fica ausente e a fonte vai para `failedSources`.

   O ano da URL sai do CVE já validado, então não há como montar outro caminho.
5. **Ransomware.** `knownRansomwareUse: true` só quando
   `knownRansomwareCampaignUse` for exatamente `Known`.
6. **Sem link para PoC.** O widget mostra só a contagem. Repositórios de PoC falsos
   que distribuem malware são um golpe conhecido, e um link a um clique do bar
   ajudaria o golpe. O link do item é a página do NVD.
7. **Rede.**
   - Só `https://`, inclusive nos redirecionamentos (no máximo 3, para o mesmo
     esquema); qualquer outro esquema é erro antes de conectar.
   - Timeouts: 5 s para conectar, 10 s por requisição; orçamento total de 20 s por
     execução (o widget desiste aos 30 s). O que não terminar até lá conta como
     falha da fonte.
   - Limite de corpo lido em streaming, sem confiar no `Content-Length`.
   - No máximo 6 requisições de PoC em paralelo.
   - `User-Agent: thedailybugle/<versão> (+https://github.com/DaviFelype/TheDailyBugle)`.
     Sem cookies, sem token.
8. **Cache.**
   - Pasta `0700`, arquivos `0600`; se já existirem mais abertos, as permissões são
     corrigidas. Escrita atômica (arquivo temporário na mesma pasta + `rename`).
     Link simbólico no lugar do arquivo ou da pasta: recusado.
   - `kev.json`: as entradas já validadas e reduzidas (CVE, nome, data, ransomware)
     dos últimos 60 dias, com o horário da busca. TTL de 6 h.
   - `poc.json`: mapa CVE → contagem e horário da busca. TTL de 6 h por CVE.
   - Cache vencido: busca de novo. Se a busca falhar, usa o cache vencido (até 7
     dias) e a fonte vai para `failedSources`.
   - Cache corrompido, ilegível ou acima de 4 MiB: tratado como ausente (é
     sobrescrito na próxima busca boa).
9. **Contrato v1, campos novos** (todos opcionais; o schema é atualizado):
   - no item: `publishedAt` (`AAAA-MM-DD`), `pocCount` (inteiro 0–10 000),
     `knownRansomwareUse` (booleano);
   - no topo: `failedSources`, lista de nomes de fonte (`PlainText`, até 10).
10. **Saída.** O binário sai com 0 mesmo quando uma fonte falha: o feed sai com o
    que deu certo e a falha em `failedSources`. Logs em stderr, sem corpo de
    resposta.
11. **Plugin.**
    - Segunda linha do item: `CISA KEV · 2026-11-03 · 3 PoC · ransomware` (cada
      parte só se o campo existir).
    - Com `failedSources` não vazio: aviso fixo no topo do popup, `Could not
      update: <fontes>`. O bar não muda de ícone (os outros itens continuam
      válidos).
    - O `Model.js` valida os campos novos como os outros: data no formato certo,
      `pocCount` inteiro dentro do limite, booleano de verdade; senão o campo some.

## Casos de teste

| ID | Cenário | Entrada | Resultado esperado | Tipo |
|---|---|---|---|---|
| T01 | KEV real | fixture `kev/catalog.json` (recorte real) | entradas válidas, na ordem de `dateAdded` | unidade |
| T02 | Janela de 30 dias | entradas de hoje, de 29, 30 e 31 dias atrás | ficam hoje, 29 e 30 | unidade |
| T03 | Limite de 100 itens | 150 entradas na janela | as 100 mais recentes | unidade |
| T04 | CVE repetido | mesmo `cveID` duas vezes | só a primeira | unidade |
| T05 | Título e URL | entrada real | título `CVE… nome`, URL do NVD montada | unidade |
| T06 | Ransomware | `Known`, `Unknown`, `known`, ausente | `true` só para `Known` | unidade |
| T07 | PoC real | fixture `poc/CVE-2026-97229.json` | `pocCount` = 7 | unidade |
| T08 | PoC 404 | resposta 404 | `pocCount` = 0, sem falha | integração (wiremock) |
| T09 | PoC com erro | 500, JSON inválido | `pocCount` ausente, `PoC-in-GitHub` em `failedSources` | integração (wiremock) |
| T10 | Cache novo | `kev.json` com 1 h | nenhuma requisição | integração (wiremock com expectativa 0) |
| T11 | Cache vencido | `kev.json` com 7 h | uma requisição, cache regravado | integração |
| T12 | Cache vencido e rede fora | `kev.json` com 7 h, servidor 503 | itens do cache, `CISA KEV` em `failedSources` | integração |
| T13 | Sem cache e rede fora | pasta vazia, servidor 503 | sem itens de KEV, `CISA KEV` em `failedSources`, saída 0 | integração |
| T14 | Cache corrompido | `kev.json` com lixo | tratado como ausente, busca de novo | integração |
| T15 | Permissões | pasta e arquivos criados | `0700` e `0600`; corrigidas se estavam `0755`/`0644` | integração |
| T16 | Escrita atômica | falha simulada no meio | cache antigo intacto | unidade |
| T17 | Binário com cache novo | `XDG_CACHE_HOME` com cache de hoje, proxy apontando para porta fechada | stdout segue o schema, `failedSources` vazio | integração (processo real) |
| T18 | Schema novo | saída com os campos novos | válida; com `pocCount` -1, data `2026-13-01`, `failedSources` com 11 nomes: inválida | unidade |
| T19 | `Model.js` campos novos | itens com e sem os campos | campos válidos mantidos, inválidos descartados | unidade (node) |
| T20 | Linha de detalhe | item com todos os campos / só `source` | `CISA KEV · 2026-11-03 · 3 PoC · ransomware` / `CISA KEV` | unidade (node) |
| T21 | Aviso de fonte | `failedSources: ["CISA KEV"]` | `Could not update: CISA KEV`; ícone do bar normal | unidade (node) |
| T22 | QML | `FeedPanel.qml` com a linha de detalhe e o aviso | `qmllint` sem avisos | integração |

## Casos de segurança

| ID | Ataque ou falha | Entrada hostil | Resultado esperado |
|---|---|---|---|
| S01 | Feed gigante | corpo de 9 MiB, com e sem `Content-Length` mentiroso | leitura para em 8 MiB, fonte falha |
| S02 | Redirecionamento para fora do HTTPS | 302 para `http://`, `file:///etc/passwd` | erro antes de seguir |
| S03 | Laço de redirecionamento | 4 redirecionamentos | erro |
| S04 | Servidor lento | resposta depois de 11 s; e 40 PoCs lentos | erro por requisição; execução termina dentro do orçamento de 20 s |
| S05 | Esquema não HTTPS na configuração | URL base `http://` no cliente de produção | erro antes de conectar |
| S06 | Caminho via CVE | `cveID` `CVE-2026-1/../../x`, `CVE-2026-1?x`, `cve-2026-1234` | entrada descartada; nenhuma URL montada |
| S07 | Texto hostil no nome | `vulnerabilityName` com bidi, ESC, `<img>`, 10 000 caracteres | limpo como `PlainText` |
| S08 | JSON muito aninhado | 100 000 `[` | erro de parse, sem estouro de pilha |
| S09 | Tipos errados | `vulnerabilities` objeto; `dateAdded` número; `html_url` `javascript:` | entrada ou feed recusado, sem pânico |
| S10 | Link simbólico no cache | `kev.json` → `~/.ssh/id_ed25519`; pasta do cache → `/tmp/x` | recusado; o alvo não é lido nem escrito |
| S11 | Cache adulterado | `kev.json` com CVE inválido ou título com bidi | mesma validação da rede |
| S12 | Pânico nos parsers | bytes arbitrários em KEV, PoC e cache | nunca entra em pânico (fuzz) |
| S13 | `Model.js` com campos hostis | `pocCount` `"3"`, `1e309`, `-1`; `publishedAt` `<b>`; `failedSources` com 1 000 nomes | campos descartados; até 10 nomes |

## Dados de teste

- `tests/fixtures/kev/catalog.json`: recorte do catálogo real (versão `2026.11.03`):
  cabeçalho + 7 entradas: cinco na janela de 30 dias contada a partir de
  2026-11-03, duas fora (uma delas com ransomware `Known`). Capturado em 2026-11-03 de
  `cisagov/kev-data`, o repositório oficial da CISA com o mesmo arquivo do feed
  (pela API do GitHub, já que o ai-jail bloqueia `cisa.gov`). As datas dos testes de
  janela são geradas a partir de "hoje" no próprio teste, para não envelhecerem.
- `tests/fixtures/poc/CVE-2026-97229.json`: arquivo real do PoC-in-GitHub
  (7 repositórios), capturado em 2026-11-03. Das 34 CVEs da janela nessa data, 21
  tinham arquivo e 13 davam 404 (sem PoC).
- Hostis (S01–S13) gerados nos testes ou escritos à mão em `tests/fixtures/kev/` e
  `tests/fixtures/poc/`.
- Termos verificados em 2026-11-03: KEV sob CC0 1.0 (sem uso do logo da CISA ou do
  DHS, sem sugerir endosso); PoC-in-GitHub sob CC0 1.0. O README ganha a atribuição.

### Dependências novas (pedido de aprovação)

| Crate | Versão | Publicada em | Uso | Por quê |
|---|---|---|---|---|
| `reqwest` | 0.13.5 | 2026-09-08 | binário | Prevista no roadmap. Sem features padrão; `rustls-no-provider` + `http2`. |
| `rustls` | 0.23.46 | 2026-10-14 | binário | Só para escolher o provedor `ring`: o padrão (`aws-lc-rs`) traz a licença OpenSSL, que o `deny.toml` não aceita, e um build em C bem maior. |
| `tokio` | 1.54.1 | 2026-10-21 | binário | Runtime do `reqwest`; features `rt`, `time`, `sync` (PoCs em paralelo com limite e orçamento total). |
| `time` | 0.3.55 | 2026-08-01 | binário | Validar `AAAA-MM-DD` e calcular a janela. Só a feature `std`. |
| `wiremock` | 0.6.5 | 2025-08-24 | só testes | Servidor HTTP falso (regra do CLAUDE.md). |
| `tempfile` | 3.27.0 | 2026-03-11 | só testes | Pastas de cache temporárias. |

O `tokio` de testes ganha `macros` e `rt-multi-thread` só em `dev-dependencies`.

## Critério de pronto

- [ ] T01–T22 e S01–S13 automatizados e passando.
- [ ] Cobertura de linhas ≥ 90%.
- [ ] CI verde, incluindo os novos alvos de fuzzing.
- [ ] Conferido por você: o popup mostra as CVEs reais do KEV com a contagem de PoC;
      com a rede desligada, mostra o cache e o aviso.
- [ ] README: fontes, atribuição e onde fica o cache.

## Riscos

- **Testes de rede sem HTTPS.** O wiremock serve `http://`. Mitigação: o cliente
  recebe a política de URL na construção; a de produção só aceita `https://` (S05
  prova), a de testes aceita também `http://127.0.0.1`. A política é testada à parte.
- **Build do `ring`** compila C e assembly no `build.rs` (dentro do sandbox). Se
  falhar no `ci / sandbox`, registro em `docs/problems/` antes de mudar de rumo.
- **PoC-in-GitHub conta repositórios, não PoCs bons.** Muitos são cópias ou falsos.
  Aceito: o número é um sinal, não uma garantia; sem link (regra 6).
- **Mudança de formato do KEV** (campo novo, renomeado). Mitigação: só os campos
  usados são lidos; os outros são ignorados; um campo usado sumindo vira entrada
  descartada e aparece na contagem em stderr.
- **`raw.githubusercontent.com` tem limite de taxa sem token.** Com cache de 6 h e
  até 100 CVEs, são no máximo ~400 requisições por dia. Se o limite aparecer, o PoC
  falha sozinho e o KEV continua.
- **Relógio da máquina errado** muda a janela e o TTL. Aceito; um cache "do futuro"
  conta como vencido.
