#!/usr/bin/env bash
# T11: the repository root is the plugin folder `omarchy plugin add` installs,
# so it must pass Omarchy's validator. Uses the vendored copy (CI has no
# Omarchy) and, when present, the one installed on this machine too.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

"$root/scripts/vendor/omarchy-plugin-validate" "$root"
echo "ok: vendored validator"

if command -v omarchy-plugin-validate >/dev/null; then
  omarchy-plugin-validate "$root"
  echo "ok: installed validator"
fi
