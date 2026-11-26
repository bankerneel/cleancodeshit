#!/usr/bin/env bats

load test_helper

seed_risks() {
  fixture "$HOME/.bun/install/cache/package"
  fixture "$HOME/.cache/puppeteer/chrome/browser"
  fixture "$HOME/.codex/sessions/2026/01/02/rollout-demo.jsonl"
  make_project demo
  export CCS_DOCKER_ENABLED=1
}

@test "safe and all exclude danger and stale, even with --in" {
  seed_risks
  run ccs --dry-run --select safe --in "$HOME/work"
  [ "$status" -eq 0 ]
  plan_only
  has "bun install cache"
  lacks "Puppeteer"
  lacks "transcripts"
  lacks "Stale project folders"
  run ccs --dry-run --select all --in "$HOME/work"
  [ "$status" -eq 0 ]
  plan_only
  has "Puppeteer"
  lacks "transcripts"
  lacks "Stale project folders"
  lacks "Stopped containers"
  lacks "Unused images"
}

@test "--yes rejects a mixed plan containing danger before anything runs" {
  seed_risks
  run ccs --select 'bun,codex_history' --yes
  [ "$status" -eq 2 ]
  has "--yes does not cover danger"
  [ -f "$HOME/.bun/install/cache/package" ]
  [ ! -d "$XDG_STATE_HOME/cleancodeshit" ]
}

@test "danger requires exact lowercase yes and does not accept pipes as a terminal" {
  seed_risks
  path="$HOME/.codex/sessions/2026/01/02/rollout-demo.jsonl"
  run ccs --select codex_history
  [ "$status" -eq 2 ]
  run ccs_confirm YES --select codex_history
  [ "$status" -eq 0 ]
  has "Cancelled"
  [ -f "$path" ]
  run ccs_confirm yes --select codex_history
  [ "$status" -eq 0 ]
  [ ! -e "$path" ]
}
