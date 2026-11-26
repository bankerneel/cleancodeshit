#!/usr/bin/env bats

load test_helper

seed_selection() {
  fixture "$HOME/.npm/_cacache/entry"
  fixture "$HOME/.npm/_npx/entry"
  fixture "$XDG_DATA_HOME/pnpm/store/v10/entry"
  fixture "$XDG_CACHE_HOME/yarn/v6/entry"
  fixture "$HOME/.bun/install/cache/entry"
  fixture "$XDG_CACHE_HOME/uv/entry"
  fixture "$XDG_CACHE_HOME/pip/wheels/entry"
  fixture "$XDG_CACHE_HOME/Homebrew/entry"
  fixture "$XDG_CACHE_HOME/ms-playwright/chromium"
}

@test "selection accepts decimal 08 and 09, ranges, names, commas, and spaces" {
  seed_selection
  run ccs --dry-run --select '08 09,2-4 npx'
  [ "$status" -eq 0 ]
  plan_only
  has "Homebrew downloads"
  has "Playwright browsers"
  has "npx downloaded tools"
  has "pnpm store"
  has "Yarn cache"
  lacks "npm cache clean"
}

@test "selection rejects malformed ranges, overflow and wildcard expansion" {
  seed_selection
  for selection in '2-1' '1-2-3' '-2' '2-' '99' '99999999999999999999999' '*'; do
    run ccs --dry-run --select "$selection"
    [ "$status" -eq 2 ]
    has "--select:"
  done
}
