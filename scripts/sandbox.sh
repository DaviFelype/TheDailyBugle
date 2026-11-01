#!/usr/bin/env bash
# Runs a command that executes third-party code (cargo build scripts, proc
# macros, fuzz targets) inside bubblewrap:
#   - no network: downloads happen first, out here, where nothing runs;
#   - the project is read-only except build output folders, so nothing can
#     plant git hooks or edit files that later run on the host;
#   - HOME, /run and /tmp are empty (no SSH keys, gh token, agent sockets);
#   - a fixed list of environment variables (no tokens).
# See docs/test-strategy/000-pipeline-ci.md, "Isolamento local".
#
# Usage: scripts/sandbox.sh <command> [args...]
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# 125, like `env` and `docker run`: the sandbox failed, not the command.
fail() {
  printf 'sandbox.sh: %s\n' "$*" >&2
  exit 125
}

(($# > 0)) || fail "usage: scripts/sandbox.sh <command> [args...]"

# The initial user namespace maps every uid to itself. Anything else means we
# are already isolated by something (ai-jail, a container), where bubblewrap
# cannot nest. Read from the kernel, so an environment variable cannot fake it.
in_initial_user_namespace() {
  local inside outside count
  read -r inside outside count </proc/self/uid_map
  [[ $inside == 0 && $outside == 0 && $count == 4294967295 ]]
}

if ! in_initial_user_namespace; then
  printf 'sandbox.sh: already inside a user namespace (ai-jail, container); running without bubblewrap\n' >&2
  exec "$@"
fi

command -v bwrap >/dev/null || fail "bubblewrap (bwrap) not found; install it first"

# `cargo fetch` only downloads; no crate code runs. Done here because the
# sandbox has no network.
if [[ $1 == cargo ]]; then
  for manifest in "$root/Cargo.toml" "$root/fuzz/Cargo.toml"; do
    if [[ -f $manifest && -f ${manifest%.toml}.lock ]]; then
      cargo fetch --locked --quiet --manifest-path "$manifest" ||
        fail "cargo fetch failed for $manifest"
    fi
  done
fi

# Tools pinned in mise.toml, resolved now: mise's own config stays hidden
# because users keep secrets in its [env] section.
path="$PATH"
if command -v mise >/dev/null; then
  mise_paths="$(cd "$root" && mise bin-paths 2>/dev/null | paste -sd: -)" || mise_paths=""
  if [[ -n $mise_paths ]]; then path="$mise_paths:$path"; fi
fi

rustup_home="${RUSTUP_HOME:-$HOME/.rustup}"
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
mise_data="${MISE_DATA_DIR:-$HOME/.local/share/mise}"

args=(
  --ro-bind / /
  --dev /dev
  --proc /proc
  --tmpfs /tmp
  --tmpfs /run
  --tmpfs /home
  --tmpfs "$HOME"
  --unshare-all
  --unshare-user
  --disable-userns
  --die-with-parent
  --new-session
  --clearenv
)

for dir in /mnt /media /srv; do
  if [[ -d $dir ]]; then args+=(--tmpfs "$dir"); fi
done

# Toolchains and the crate cache, read-only. Not all of CARGO_HOME: it can
# hold credentials.toml and config.toml with registry tokens.
for dir in "$rustup_home" "$cargo_home/bin" "$cargo_home/registry" "$cargo_home/git" "$mise_data"; do
  if [[ -d $dir ]]; then args+=(--ro-bind "$dir" "$dir"); fi
done

args+=(--ro-bind "$root" "$root")
mkdir -p "$root/target"
writable=("$root/target")
if [[ -d $root/fuzz ]]; then
  mkdir -p "$root/fuzz/target" "$root/fuzz/corpus" "$root/fuzz/artifacts" "$root/fuzz/coverage"
  writable+=("$root/fuzz/target" "$root/fuzz/corpus" "$root/fuzz/artifacts" "$root/fuzz/coverage")
fi
for dir in "${writable[@]}"; do
  args+=(--bind "$dir" "$dir")
done

args+=(
  --setenv PATH "$path"
  --setenv HOME "$HOME"
  --setenv RUSTUP_HOME "$rustup_home"
  --setenv CARGO_HOME "$cargo_home"
  --setenv CARGO_NET_OFFLINE true
)
for name in USER LANG LC_ALL TERM CI RUSTUP_TOOLCHAIN CARGO_TERM_COLOR RUST_BACKTRACE NEXTEST_PROFILE; do
  if [[ -n ${!name:-} ]]; then args+=(--setenv "$name" "${!name}"); fi
done

case "$PWD/" in
  "$root"/*) workdir="$PWD" ;;
  *) workdir="$root" ;;
esac
args+=(--chdir "$workdir")

exec bwrap "${args[@]}" -- "$@"
