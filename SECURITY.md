# Security Policy

## Reporting a vulnerability

Please **do not** open a public issue.

Report it privately through GitHub:
[Security → Report a vulnerability](https://github.com/DaviFelype/TheDailyBugle/security/advisories/new).

Include what you can: affected version or commit, steps to reproduce, and impact.
You will get an answer within 7 days. Once a fix is released, the advisory is
published with credit to you, unless you prefer to stay anonymous.

## Supported versions

Until 1.0.0, only the latest release gets security fixes.

## Scope

In scope:

- the `thedailybugle` binary, including how it fetches, parses, caches and
  sanitizes data from external sources;
- the QML plugin, including how it renders text and opens links;
- the release pipeline (GitHub Actions workflows, release artifacts).

Out of scope: vulnerabilities in the upstream data sources themselves. Report those
to their maintainers.
