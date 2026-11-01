#!/usr/bin/env bash
# Tests for scripts/sandbox.sh: T13-T16 and S13-S19 in
# docs/test-strategy/000-pipeline-ci.md.
#
# Inside another user namespace (ai-jail, containers) bubblewrap cannot
# nest, so the isolation cases are skipped there. In CI (CI=true) a skip
# counts as a failure: the isolation must be proven somewhere.
set -uo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
sandbox="$root/scripts/sandbox.sh"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"; rm -f "$root/sandbox-probe" "$root/.git/hooks/sandbox-probe" "$root/target/sandbox-probe"' EXIT

failures=0
skips=0
pass() { printf 'ok   %s\n' "$1"; }
fail() { printf 'FAIL %s\n' "$1"; failures=$((failures + 1)); }
skip() { printf 'skip %s\n' "$1"; skips=$((skips + 1)); }
check() { local name="$1"; shift; if "$@"; then pass "$name"; else fail "$name"; fi; }

in_initial_user_namespace() {
  local inside outside count
  read -r inside outside count </proc/self/uid_map
  [[ $inside == 0 && $outside == 0 && $count == 4294967295 ]]
}

# --- Always testable -------------------------------------------------------

exit_code_is_kept() {
  "$sandbox" bash -c 'exit 7' 2>/dev/null
  [[ $? == 7 ]]
}
check "T14 exit code of the command is kept" exit_code_is_kept

runs_directly_inside_a_namespace() {
  local status
  if in_initial_user_namespace; then
    unshare --user --map-current-user "$sandbox" bash -c 'exit 7' 2>"$scratch/stderr"
  else
    "$sandbox" bash -c 'exit 7' 2>"$scratch/stderr"
  fi
  status=$?
  [[ $status == 7 ]] && grep -q "already inside a user namespace" "$scratch/stderr"
}
check "T15 inside a user namespace it runs the command directly and warns" runs_directly_inside_a_namespace

# --- Needs bubblewrap ------------------------------------------------------

if ! in_initial_user_namespace || ! command -v bwrap >/dev/null; then
  for name in "T13 tests pass in the sandbox" "T16 target/ is writable" \
    "S13 .git is read-only" "S14 no network" "S15 credentials in HOME are hidden" \
    "S16 secrets in the environment are dropped" "S17 writes outside the project do not persist" \
    "S18 the rest of the project is read-only" "S19 host sockets in /run are hidden"; do
    skip "$name (needs bubblewrap outside a user namespace)"
  done
else
  tests_pass() { "$sandbox" cargo nextest run --no-fail-fast >"$scratch/nextest" 2>&1; }
  check "T13 tests pass in the sandbox" tests_pass

  target_is_writable() {
    mkdir -p "$root/target"
    "$sandbox" bash -c 'touch target/sandbox-probe' 2>/dev/null && [[ -e $root/target/sandbox-probe ]]
  }
  check "T16 target/ is writable" target_is_writable

  git_is_read_only() {
    ! "$sandbox" bash -c 'echo pwned > .git/hooks/sandbox-probe' 2>/dev/null &&
      [[ ! -e $root/.git/hooks/sandbox-probe ]]
  }
  check "S13 .git is read-only" git_is_read_only

  no_network() {
    ! "$sandbox" timeout 10 bash -c 'exec 3<>/dev/tcp/1.1.1.1/443' 2>/dev/null &&
      ! "$sandbox" timeout 10 getent hosts github.com >/dev/null 2>&1
  }
  check "S14 no network" no_network

  credentials_are_hidden() {
    local fake_home="$scratch/home"
    mkdir -p "$fake_home/.ssh" "$fake_home/.config/gh"
    echo secret >"$fake_home/.ssh/id_ed25519"
    echo secret >"$fake_home/.config/gh/hosts.yml"
    HOME="$fake_home" "$sandbox" bash -c \
      'test ! -e "$HOME/.ssh/id_ed25519" && test ! -e "$HOME/.config/gh/hosts.yml"'
  }
  check "S15 credentials in HOME are hidden" credentials_are_hidden

  environment_secrets_are_dropped() {
    GH_TOKEN=fake-token AWS_SECRET_ACCESS_KEY=fake-key "$sandbox" bash -c \
      '[[ -z ${GH_TOKEN:-} && -z ${AWS_SECRET_ACCESS_KEY:-} ]]'
  }
  check "S16 secrets in the environment are dropped" environment_secrets_are_dropped

  writes_outside_do_not_persist() {
    local fake_home="$scratch/home2"
    mkdir -p "$fake_home"
    HOME="$fake_home" "$sandbox" bash -c \
      "touch \"\$HOME/escape\" /tmp/sandbox-escape-$$" 2>/dev/null
    [[ ! -e $fake_home/escape && ! -e /tmp/sandbox-escape-$$ ]]
  }
  check "S17 writes outside the project do not persist" writes_outside_do_not_persist

  project_is_read_only() {
    ! "$sandbox" bash -c 'touch sandbox-probe' 2>/dev/null && [[ ! -e $root/sandbox-probe ]]
  }
  check "S18 the rest of the project is read-only" project_is_read_only

  run_is_hidden() { "$sandbox" bash -c 'test -z "$(ls -A /run)"'; }
  check "S19 host sockets in /run are hidden" run_is_hidden
fi

if [[ ${CI:-} == true && $skips -gt 0 ]]; then
  echo "error: $skips case(s) skipped in CI; the sandbox must be tested here" >&2
  failures=$((failures + skips))
fi

printf '\n%d failed, %d skipped\n' "$failures" "$skips"
[[ $failures == 0 ]]
