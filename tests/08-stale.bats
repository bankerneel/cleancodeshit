#!/usr/bin/env bats

load test_helper

@test "stale roots refuse slash, HOME, escaped folders, and .git" {
  make_project demo
  mkdir -p -- "$HOME/work/.git/deep" "$CASE_ROOT/outside"
  for root in / "$HOME" "$HOME/work/.git/deep" "$CASE_ROOT/outside"; do
    run ccs --list --in "$root"
    [ "$status" -eq 2 ]
  done
  [ -d "$HOME/work/demo/node_modules" ]
}

@test "stale selection lists age and size and does not touch fresh or unrecognized projects" {
  make_project 'old one'
  make_project fresh fresh
  fixture "$HOME/work/no-context/node_modules/package/index"
  fixture "$HOME/work/old one/.git/node_modules/internal"
  fixture "$HOME/work/old one/build/source.c"
  old "$HOME/work/old one/build/source.c"
  run ccs --list --in "$HOME/work" --older-than 08
  [ "$status" -eq 0 ]
  has "old\\ one/node_modules"
  has "day(s) since source changed"
  lacks "fresh/node_modules"
  lacks "no-context/node_modules"
  run ccs --select stale --yes --in "$HOME/work"
  [ "$status" -eq 0 ]
  [ ! -d "$HOME/work/old one/node_modules" ]
  [ -f "$HOME/work/old one/.git/node_modules/internal" ]
  [ -f "$HOME/work/old one/build/source.c" ]
  [ -d "$HOME/work/fresh/node_modules" ]
  [ -f "$HOME/work/no-context/node_modules/package/index" ]
}

@test "stale selection never crosses symlinks and excludes embedded .git directories" {
  make_project old
  fixture "$HOME/work/old/node_modules/dependency/.git/config"
  fixture "$HOME/external/old/package.json"
  fixture "$HOME/external/old/.next/cache"
  old "$HOME/external/old/package.json"
  link_dir "$HOME/external" "$HOME/work/link" || skip "directory symlinks unavailable"
  run ccs --list --in "$HOME/work/link"
  [ "$status" -eq 2 ]
  run ccs --list --in "$HOME/work"
  [ "$status" -eq 0 ]
  lacks "old/.next"
  lacks "old/node_modules"
  [ -f "$HOME/work/old/node_modules/dependency/.git/config" ]
}

@test "stale age accepts zero but rejects malformed and excessive thresholds" {
  make_project demo
  run ccs --list --in "$HOME/work" --older-than 0
  [ "$status" -eq 0 ]
  has "demo/node_modules"
  for days in -1 365001 typo; do
    run ccs --list --in "$HOME/work" --older-than "$days"
    [ "$status" -eq 2 ]
  done
  run ccs --list --older-than 30
  [ "$status" -eq 2 ]
}

@test "stale scanner preserves tracked artifacts and checks activity in authored build trees" {
  make_project demo
  fixture "$PROJECT_ROOT/build/source.c"
  run ccs --list --in "$HOME/work"
  [ "$status" -eq 0 ]
  lacks 'demo/node_modules'
  old "$PROJECT_ROOT/build/source.c"
  git -C "$PROJECT_ROOT" init -q
  git -C "$PROJECT_ROOT" add -f -- node_modules/demo/index.js
  run ccs --list --in "$HOME/work"
  [ "$status" -eq 0 ]
  lacks 'demo/node_modules'
  [ -f "$PROJECT_ROOT/node_modules/demo/index.js" ]
}

@test "stale scanner checks the ancestor index of a nested monorepo package" {
  make_project monorepo/apps/web
  repo="$HOME/work/monorepo"
  git -C "$repo" init -q
  git -C "$repo" add -f -- apps/web/node_modules/demo/index.js
  run ccs --list --in "$HOME/work"
  [ "$status" -eq 0 ]
  lacks 'apps/web/node_modules'
  run ccs --list --in "$repo/apps"
  [ "$status" -eq 2 ]
  has 'choose the Git repository root'
  [ -f "$PROJECT_ROOT/node_modules/demo/index.js" ]
}
