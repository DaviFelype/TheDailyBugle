# TheDailyBugle

A bar widget for [Omarchy](https://omarchy.org) 4 (Quattro shell) that shows a fast
cybersecurity feed:

- newly exploited vulnerabilities that have a public proof of concept;
- major ransomware attacks, worldwide and in Brazil;
- relevant security news of the day.

> **Status:** early development. Nothing is installable yet. See
> [docs/ROADMAP.md](docs/ROADMAP.md) (in Portuguese).

## Architecture

```
┌──────────────────────────────┐   JSON on stdout    ┌──────────────────────────────┐
│ Rust binary `thedailybugle`  │ ──────────────────► │ QML plugin (Quattro shell)   │
│ - fetches every source       │ `thedailybugle      │ - bar icon + popup panel     │
│ - validates and sanitizes    │    feed --json`     │ - pure display logic in      │
│ - disk cache                 │                     │   Model.js                   │
└──────────────────────────────┘                     └──────────────────────────────┘
```

Everything that comes from the internet is treated as untrusted input. It is
validated and sanitized in the Rust binary before it reaches the QML plugin, and
the plugin renders all text as plain text. See
[ADR-0002](docs/adr/0002-binary-plus-plugin.md).

| Item | Value |
|---|---|
| Plugin ID | `davifelype.thedailybugle` |
| Binary | `thedailybugle` |
| Config | `~/.config/thedailybugle/config.toml` |

## Development

Requirements: [mise](https://mise.jdx.dev), `git` and a C compiler.

```sh
scripts/bootstrap.sh   # installs the pinned toolchain and the git hooks
```

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

## Security

Please report vulnerabilities privately. See [SECURITY.md](SECURITY.md).

## License

[MIT](LICENSE)

<!-- gate S04 -->
