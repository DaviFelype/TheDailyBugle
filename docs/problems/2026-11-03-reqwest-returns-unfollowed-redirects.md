# 2026-11-03 — reqwest devolve como resposta comum o redirecionamento que não segue

- **Status:** resolvido
- **Commits/PRs:** 4f596e0 (`feat(sources): ...`), teste em 19422a4

## Sintoma

O teste S02 (`redirects_leaving_the_policy_are_refused`) falhou só no caso do
`302` com `Location: file:///etc/passwd`:

```
/to-file: Ok(HttpResponse { status: 302, body: [] })
```

O redirecionamento para `http://localhost` (host fora da política) dava
`Err(Redirect)` como esperado.

## Causa raiz

O reqwest só chama a política de redirecionamento para destinos `http`/`https`.
Para outro esquema ele nem tenta seguir e devolve o próprio `302` como resposta
final. O arquivo nunca foi lido, mas o `get` dizia `Ok`, e a fonte só falhava
depois, por causa do status diferente de 200.

## Solução

`HttpClient::get` trata qualquer status 3xx que chega ao fim como
`FetchError::Redirect`: se chegou até nós, é um redirecionamento que não foi
seguido.

## Como evitar

O teste S02 cobre os dois casos (outro host e `file:`). Ao trocar de cliente HTTP,
não presumir que a política de redirecionamento vê todo `Location`.
