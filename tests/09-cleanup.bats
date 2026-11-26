#!/usr/bin/env bats

load test_helper

@test "safe package cleanup uses only allowed direct entries and records real dispatch" {
  fixture "$XDG_CACHE_HOME/yarn/v6/package"
  fixture "$XDG_CACHE_HOME/yarn/keep.txt"
  fixture "$XDG_CACHE_HOME/pip/wheels/package"
  fixture "$XDG_CACHE_HOME/uv/package"
  fixture "$XDG_CACHE_HOME/go-build/package"
  fixture "$XDG_CACHE_HOME/pre-commit/db.db"
  printf 'This directory is maintained by the pre-commit project.\n' > "$XDG_CACHE_HOME/pre-commit/README"
  fixture "$XDG_CACHE_HOME/pre-commit/repo12345678/example"
  fixture "$XDG_CACHE_HOME/pre-commit/keep.txt"
  export CCS_DOCKER_ENABLED=1
  run ccs --select 'yarn pip uv go precommit dk_build' --yes
  [ "$status" -eq 0 ]
  [ ! -d "$XDG_CACHE_HOME/yarn/v6" ] && [ -f "$XDG_CACHE_HOME/yarn/keep.txt" ]
  [ ! -d "$XDG_CACHE_HOME/pip/wheels" ]
  [ ! -f "$XDG_CACHE_HOME/pre-commit/db.db" ] && [ -f "$XDG_CACHE_HOME/pre-commit/keep.txt" ]
  grep -F $'\tcache\tprune' "$CCS_CLEANUP_LOG"
  grep -F $'go\tclean\t-cache' "$CCS_CLEANUP_LOG"
  grep -Fx $'docker\t--context\tsample\tbuilder\tprune\t-af' "$CCS_CLEANUP_LOG"
  [ -d "$XDG_STATE_HOME/cleancodeshit" ]
}

@test "installer verifies release hash and leaves the installed version on mismatch" {
  assert_fake_home
  mkdir -p -- "$CASE_ROOT/release"
  cp -- "$PRODUCT" "${PRODUCT%/*}/install.sh" "${PRODUCT%/*}/cleancodeshit.sha256" "$CASE_ROOT/release/"
  run "${BASH_UNDER_TEST:-bash}" "$CASE_ROOT/release/install.sh"
  [ "$status" -eq 0 ]
  has "Verified SHA-256"
  [ -f "$HOME/.local/bin/cleancodeshit" ]
  echo 'modified fixture' >> "$CASE_ROOT/release/cleancodeshit"
  run "${BASH_UNDER_TEST:-bash}" "$CASE_ROOT/release/install.sh"
  [ "$status" -eq 1 ]
  has "Checksum mismatch"
  cmp "$PRODUCT" "$HOME/.local/bin/cleancodeshit"
}

@test "Gradle override cleans its caches with the default Gradle home preserved" {
  export GRADLE_USER_HOME="$HOME/other-gradle"
  fixture "$GRADLE_USER_HOME/caches/fixture"
  fixture "$GRADLE_USER_HOME/gradle.properties"
  fixture "$HOME/.gradle/caches/keep"
  run ccs --select gradle --yes
  [ "$status" -eq 0 ]
  [ ! -d "$GRADLE_USER_HOME/caches" ]
  [ -f "$GRADLE_USER_HOME/gradle.properties" ]
  [ -f "$HOME/.gradle/caches/keep" ]
}

@test "npm cache cleanup stops before dispatch when its subtree crosses a mount" {
  export CCS_FAKE_MOUNT="$HOME/.npm/_cacache/mounted"
  fixture "$CCS_FAKE_MOUNT/keep"
  run ccs --select npm --yes
  [ "$status" -eq 1 ]
  [ -f "$CCS_FAKE_MOUNT/keep" ]
  ! grep -F $'\tcache\tclean' "$CCS_CLEANUP_LOG"
  grep -F 'Cache path is mounted' "$XDG_STATE_HOME/cleancodeshit"/cleancodeshit-*
}

interrupt_cleanup() (
  assert_fake_home || return 1
  export CCS_INTERRUPT_PIDS="$CASE_ROOT/interrupt-pids"
  parent= worker= sleeper=
  trap 'for pid in "$parent" "$worker" "$sleeper"; do [ -z "$pid" ] || kill -KILL "$pid" 2>/dev/null || :; done' EXIT
  set -m
  "${BASH_UNDER_TEST:-bash}" "$PRODUCT" --select npm --yes >"$CASE_ROOT/interrupt-console" 2>&1 &
  parent=$!
  for ((i=0; i<120; i++)); do
    [ ! -s "$CCS_INTERRUPT_PIDS" ] || break
    sleep 0.05
  done
  read -r worker sleeper < "$CCS_INTERRUPT_PIDS" || return 1
  kill -INT "$parent" || return 1
  rc=0
  wait "$parent" || rc=$?
  parent=
  [ "$rc" -eq 130 ] || return 1
  grep -F 'Interrupted.' "$CASE_ROOT/interrupt-console" || return 1
  for pid in "$worker" "$sleeper"; do
    if kill -0 "$pid" 2>/dev/null; then
      case "$CCS_REAL_UNAME" in
        Linux|Darwin) [[ "$("$CCS_REAL_PS" -o stat= -p "$pid")" == Z* ]] || return 1 ;;
        *) return 1 ;;
      esac
    fi
  done
  worker= sleeper=
)

@test "Ctrl+C stops the active cleanup and its child process" {
  fixture "$HOME/.npm/_cacache/fixture"
  run interrupt_cleanup
  [ "$status" -eq 0 ]
}
