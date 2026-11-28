#!/usr/bin/env bash
set -eu

fail() {
  printf '%s\n' "$*" >&2
  exit 1
}

[ "$#" -eq 1 ] || fail 'Usage: scripts/release.sh X.Y.Z'
repo=$(CDPATH='' cd -P -- "$(dirname -- "$0")/.." && pwd -P)
cd "$repo"
[ "$(git symbolic-ref --quiet --short HEAD || :)" = main ] || fail 'Run this from main.'
status=$(git status --porcelain --untracked-files=all)
[ -z "$status" ] || fail 'The working tree must be clean.'
command -v node >/dev/null 2>&1 || fail 'Node.js 22 or newer is required.'
exec node scripts/release-metadata.mjs bump "$1"
