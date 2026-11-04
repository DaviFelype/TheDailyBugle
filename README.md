# TheDailyBugle

A bar widget for [Omarchy](https://omarchy.org) 4 (Quattro shell) that shows a fast
cybersecurity feed:

- newly exploited vulnerabilities that have a public proof of concept;
- major ransomware attacks, worldwide and in Brazil;
- relevant security news of the day.

> **Status:** early development. Vulnerabilities come from real sources;
> ransomware and news are still built-in sample items. See
> [docs/ROADMAP.md](docs/ROADMAP.md) (in Portuguese).

## Sources

| Source | What it gives | License |
|---|---|---|
| [CISA Known Exploited Vulnerabilities](https://www.cisa.gov/known-exploited-vulnerabilities-catalog) | CVEs added in the last 30 days, and whether they are used by ransomware | CC0 1.0 |
| [nomi-sec/PoC-in-GitHub](https://github.com/nomi-sec/PoC-in-GitHub) | how many public GitHub repositories claim a proof of concept for each CVE | CC0 1.0 |

TheDailyBugle is not affiliated with or endorsed by CISA or DHS. Each
vulnerability links to its NVD page. The widget shows only the number of PoC
repositories, never links to them: fake PoC repositories that spread malware
are a known trick.

When a source cannot be reached, the feed keeps what worked, lists the source in
`failedSources` and the popup says `Could not update: <source>`.

## Cache

Fetched data is kept in `$XDG_CACHE_HOME/thedailybugle/` (by default
`~/.cache/thedailybugle/`), with `0700` on the folder and `0600` on the files,
for 6 hours. If a source fails, cached data up to 7 days old is shown instead.
Deleting the folder is always safe.

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
cargo install --path . --locked --root ~/.local      # puts thedailybugle in ~/.local/bin
omarchy plugin add ~/path/to/TheDailyBugle --enable  # clones the committed HEAD
```

The shell runs `thedailybugle` from the `PATH` of the graphical session, which
Omarchy builds from the system paths, mise shims and `~/.local/bin`, but not
`~/.cargo/bin`. That is why the install goes to `~/.local`. If the popup says
"Could not load the feed", check `ls ~/.local/bin/thedailybugle`, then middle
click the icon to refresh.

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
- Optional, per item: `publishedAt` (`YYYY-MM-DD`, UTC), `pocCount` (0 to
  10000) and `knownRansomwareUse` (boolean).
- Optional, at the top: `failedSources`, up to 10 source names.

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
