#!/usr/bin/env bash
# T12: lint the plugin's QML against Omarchy's own UI types (`qs.Ui`), with
# zero warnings allowed. Quickshell exposes the shell folder as the `qs`
# module, so a temporary import root links `qs` to it.
#
# OMARCHY_SHELL_DIR: Omarchy's shell/ folder (CI checks out a pinned commit).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
shell_dir="${OMARCHY_SHELL_DIR:-/usr/share/omarchy/shell}"
qmllint="${QMLLINT:-/usr/lib/qt6/bin/qmllint}"

[[ -d $shell_dir/Ui ]] || { echo "error: no Omarchy shell at $shell_dir" >&2; exit 1; }

import_root="$(mktemp -d)"
trap 'rm -rf "$import_root"' EXIT
ln -s "$(cd "$shell_dir" && pwd)" "$import_root/qs"

"$qmllint" --max-warnings 0 -I "$import_root" "$root"/omarchy/*.qml
echo "ok: qmllint"
