#!/usr/bin/env bash
set -eu

fail() {
  printf '%s\n' "$*" >&2
  exit 1
}

[ "$#" -eq 1 ] || fail 'Usage: scripts/release.sh X.Y.Z'
repo=$(CDPATH='' cd -P -- "$(dirname -- "$0")/.." && pwd -P)
cd "$repo"
branch=$(git symbolic-ref --quiet --short HEAD || :)
if [ "$branch" != main ]; then
  origin_main=$(git rev-parse --verify --quiet refs/remotes/origin/main || :)
  if [ -n "$branch" ] || [ -z "$origin_main" ] || [ "$(git rev-parse HEAD)" != "$origin_main" ]; then
    fail 'Run this from main or with a detached HEAD at origin/main.'
  fi
fi
status=$(git status --porcelain --untracked-files=all)
[ -z "$status" ] || fail 'The working tree must be clean.'
command -v node >/dev/null 2>&1 || fail 'Node.js 22 or newer is required.'
exec node scripts/release-metadata.mjs bump "$1"
