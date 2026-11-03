# TheDailyBugle

A bar widget for [Omarchy](https://omarchy.org) 4 (Quattro shell) that shows a fast
cybersecurity feed:

- newly exploited vulnerabilities that have a public proof of concept;
- major ransomware attacks, worldwide and in Brazil;
- relevant security news of the day.

> **Status:** early development. The feed shows built-in sample items only;
> real sources come next. See [docs/ROADMAP.md](docs/ROADMAP.md) (in Portuguese).

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

## Try it locally

```sh
cargo install --path . --locked                    # puts thedailybugle in ~/.cargo/bin
omarchy plugin add ~/path/to/TheDailyBugle --enable  # clones the committed HEAD
```

The shell runs `thedailybugle` from its own `PATH`. If the bar shows an alert
glyph and the popup says the binary was not found, make sure `~/.cargo/bin` is on
the `PATH` of your graphical session.

In the bar: left click opens the popup, middle click refreshes. Clicking an item
opens it in the browser through `xdg-open`, only for `https://` links.

## JSON contract

`thedailybugle feed --json` prints one line of JSON, described by
[schema/feed-v1.schema.json](schema/feed-v1.schema.json):

```json
{"schemaVersion":1,"items":[{"id":"sample:news","category":"news",
  "title":"[Sample] Security news of the day",
  "url":"https://example.com/thedailybugle/sample/news","source":"TheDailyBugle sample"}]}
```

- `category` is `vulnerability`, `ransomware` or `news`.
- `title` (up to 300 characters) and `source` (up to 64) are one line of plain
  text, without control or bidi characters.
- `url` is `https://`, without a user name or password, up to 2048 bytes.
- At most 200 items.

The plugin refuses any other `schemaVersion` and ignores fields it does not know.

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
