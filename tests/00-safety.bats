#!/usr/bin/env bats

load test_helper

@test "the harness refuses an external HOME and the temporary root itself" {
  HOME="$CASE_ROOT"
  run ccs --version
  [ "$status" -eq 1 ]
  has "Refusing to run"
  HOME="$TESTS_DIR"
  run ccs --version
  [ "$status" -eq 1 ]
}

@test "cache scans reject override paths outside HOME and paths with traversals" {
  fixture "$CASE_ROOT/outside/chrome/keep"
  fixture "$HOME/.cache/puppeteer/chrome/local"
  export PUPPETEER_CACHE_DIR="$CASE_ROOT/outside"
  run ccs --select puppeteer --yes
  [ "$status" -eq 2 ]
  [ -f "$CASE_ROOT/outside/chrome/keep" ]
  export PUPPETEER_CACHE_DIR="$HOME/.cache/../.cache/puppeteer"
  run ccs --select puppeteer --yes
  [ "$status" -eq 2 ]
  [ -f "$HOME/.cache/puppeteer/chrome/local" ]
}

@test "symlinked cache ancestors are rejected" {
  fixture "$CASE_ROOT/outside/chrome/keep"
  link_dir "$CASE_ROOT/outside" "$HOME/link" || skip "directory symlinks unavailable"
  export PUPPETEER_CACHE_DIR="$HOME/link"
  run ccs --select puppeteer --yes
  [ "$status" -eq 2 ]
  [ -f "$CASE_ROOT/outside/chrome/keep" ]
}

@test "cleanup refuses to write a log through a symlink" {
  fixture "$HOME/.bun/install/cache/keep"
  mkdir -p -- "$CASE_ROOT/other-logs"
  link_dir "$CASE_ROOT/other-logs" "$HOME/log-link" || skip "directory symlinks unavailable"
  export XDG_STATE_HOME="$HOME/log-link"
  run ccs --select bun --yes
  [ "$status" -eq 1 ]
  [ -f "$HOME/.bun/install/cache/keep" ]
  [ ! -d "$CASE_ROOT/other-logs/cleancodeshit" ]
}
