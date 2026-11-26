#!/usr/bin/env bats

load test_helper

@test "list and dry-run report sizes and leave the fake HOME unchanged" {
  fixture "$HOME/.bun/install/cache/package"
  fixture "$HOME/.codex/log/codex-tui.log"
  make_project demo
  before=$(snapshot)
  run ccs --list --in "$HOME/work"
  [ "$status" -eq 0 ]
  has "Package managers"
  has "AI coding agents"
  has "demo/node_modules"
  lacks "> Plan"
  run ccs --dry-run --select safe
  [ "$status" -eq 0 ]
  has "> Plan"
  has "Dry run: nothing was changed."
  [ "$(snapshot)" = "$before" ]
  [ ! -d "$XDG_STATE_HOME/cleancodeshit" ]
}

@test "empty home and unsupported-platform dry runs finish without cleanup" {
  run ccs --dry-run --select safe
  [ "$status" -eq 0 ]
  has "Nothing to clean was found."
  has "Dry run: nothing was changed."
  export CCS_FAKE_UNAME=MINGW64_NT
  run ccs --list
  [ "$status" -eq 0 ]
  has "This system was skipped."
  run ccs --select safe --yes
  [ "$status" -eq 1 ]
}
