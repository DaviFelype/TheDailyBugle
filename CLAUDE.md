# CLAUDE.md

Rules for AI agents working in this repository. Keep this file under 100 lines.

## Before anything
- Read `docs/ROADMAP.md`. Decisions in section 7 are final unless the user reopens them.
- Talk to the user in Brazilian Portuguese. Code, comments, commits and this file are in English.
- Process docs (`docs/ROADMAP.md`, `docs/adr/`, `docs/test-strategy/`, `docs/problems/`, `CONTRIBUTING.md`) are in pt-BR.

## Process (TDD, mandatory)
1. Write `docs/test-strategy/NNN-<topic>.md` from `docs/test-strategy/_template.md`.
2. Stop and wait for the user to approve it. Never write product code before that.
3. Commit failing tests: `test(scope): ...` (RED).
4. Commit the minimal implementation: `feat(scope): ...` or `fix(scope): ...` (GREEN).
5. Commit cleanups separately: `refactor(scope): ...`.
6. Open a PR; merge only with green CI, using rebase (linear history).

## Git
- Never commit to `main`. Branches: `feat/`, `fix/`, `chore/`, `docs/`, `test/`, `ci/`.
- Conventional Commits, checked by `cog verify`. Extra type: `security`.
- Never bypass hooks (`--no-verify`). Fix the cause instead.
- Commit only when the user asks or the approved plan says so.

## Rust style
- Edition 2024, toolchain pinned in `rust-toolchain.toml`.
- `cargo fmt` and `cargo clippy --all-targets -- -D warnings` must pass.
- No `unwrap()`/`expect()`/`panic!` outside tests. Return `Result` with typed errors.
- No `unsafe`. Small modules with one job each; pure functions where possible.
- Name things for what they do. No abbreviations except well-known ones (`url`, `id`).

## Comments
- Explain *why*, not *what*. No commented-out code. No TODOs without an issue link.

## Tests
- Tests never touch the internet. Use `wiremock` and fixtures in `tests/fixtures/`.
- One behavior per test; the test name says the behavior.
- Every bug fix starts with a test that reproduces it.
- Snapshot tests (`insta`) only for output contracts; review snapshot diffs by hand.

## Dependencies
- Ask before adding a crate or npm package. Justify it in the PR.
- Pin exact versions. Only use releases published at least 7 days ago.
- `cargo deny check` and `cargo audit` must pass.

## Security (product rules, each one has tests)
- All external content is untrusted: validate, length-limit and strip markup in Rust.
- HTTPS only, with timeouts and a response size limit.
- QML renders external text with `Text.PlainText`, never `RichText`.
- Links open only if they are `https://`, via `xdg-open`, never through a shell.
- Cache and config files are `0600`, their directories `0700`.
- The widget never crashes the bar: every failure becomes an error state.
- No telemetry.

## Logging
- Log to stderr; stdout is reserved for the JSON contract.
- Never log tokens, API keys or full response bodies.

## Problems
- When something breaks in a non-obvious way, record it in `docs/problems/`
  using `docs/problems/_template.md`, and link the commits or PRs.

## Environment
- Agents run inside ai-jail: network is filtered and SSH is unavailable.
  Push over HTTPS (`gh auth git-credential`). Tools come from the host's mise
  installs, read-only; if one is missing, ask the user to run `scripts/bootstrap.sh`.
