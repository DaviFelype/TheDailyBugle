#!/usr/bin/env bash
# Installs every developer tool pinned in mise.toml and rust-toolchain.toml.
# Safe to run again: each step skips work that is already done.
#
# Run it on the host, not inside ai-jail. The sandbox blocks the download
# hosts, and it maps the host's mise and rustup installs read-only.
set -euo pipefail

# Nightly is only used by cargo-fuzz. Pinned so fuzzing is reproducible.
readonly FUZZ_NIGHTLY="nightly-2026-10-15"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

info() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
fail() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

for cmd in mise git cc bwrap; do
  command -v "$cmd" >/dev/null || fail "'$cmd' not found; install it first"
done

if ! curl -fsS -m 10 -o /dev/null https://static.rust-lang.org/dist/channel-rust-stable.toml.sha256; then
  fail "cannot reach static.rust-lang.org (running inside ai-jail?)"
fi

info "Trusting mise.toml"
mise trust --quiet "$root/mise.toml"

# Installers first, so the cargo and pypi backends can use them.
info "Installing Rust, uv and cargo-binstall"
mise install rust uv cargo-binstall

info "Installing the Rust toolchain from rust-toolchain.toml"
mise exec -- rustup toolchain install

info "Installing $FUZZ_NIGHTLY for cargo-fuzz"
mise exec -- rustup toolchain install "$FUZZ_NIGHTLY" --profile minimal

info "Installing the remaining tools"
mise install

info "Installed versions"
mise exec -- bash -c '
  set -e
  rustc --version
  cargo --version
  cargo clippy --version
  cargo fmt --version
  cargo nextest --version | head -n1
  cargo audit --version
  cargo deny --version
  cargo llvm-cov --version
  cargo fuzz --version
  cargo machete --version
  cog --version
  lefthook version
  gitleaks version
  node --version
  actionlint -version | head -n1
  zizmor --version
'

if [[ -f lefthook.yml ]]; then
  info "Installing git hooks"
  mise exec -- lefthook install
fi

info "Done"
