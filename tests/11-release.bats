#!/usr/bin/env bats

load test_helper

set_release_notes() {
  node --input-type=module - "$notes" <<'JS'
import { readFileSync, writeFileSync } from 'node:fs';
const file = 'CHANGELOG.md';
const source = readFileSync(file, 'utf8').replace(/^## \[Unreleased\]\n[\s\S]*?(?=^## \[)/m, '');
writeFileSync(file, source.replace(/^## \[/m, () => process.argv[2] + '## ['));
JS
}

release_repo() {
  assert_fake_home || return 1
  repo="$CASE_ROOT/repo"
  mkdir -p "$repo"
  local source="${TESTS_DIR%/*}" patch
  cp -R "$source"/{cleancodeshit,cleancodeshit.sha256,CHANGELOG.md,README.md,SECURITY.md,docs,packaging,scripts,site,tools} "$repo/"
  mkdir -p "$repo/tests"
  cp -R "$TESTS_DIR"/{check-site.mjs,shims} "$repo/tests/"
  cd "$repo"
  git init -q -b main
  git config user.name 'Release fixture'
  git config user.email 'release@example.invalid'
  git config core.autocrlf false
  current=$(sed -n 's/^VERSION="\(.*\)"$/\1/p' cleancodeshit)
  patch=${current##*.}
  next="${current%.*}.$((patch + 1))"
  set_release_notes
  git add .
  git commit -qm 'Release fixture'
}

release() {
  "${BASH_UNDER_TEST:-bash}" scripts/release.sh "$@"
}

@test "release refuses a dirty tree and a branch other than main" {
  notes=$'## [Unreleased]\n\n- Sample change.\n\n'
  release_repo
  touch untracked
  run release "$next"
  [ "$status" -ne 0 ]
  has 'working tree must be clean'
  rm untracked
  git update-ref refs/remotes/origin/main HEAD
  git switch -qc other
  run release "$next"
  [ "$status" -ne 0 ]
  has 'Run this from main'
  [ -z "$(git status --porcelain)" ]
}

@test "release accepts only a clean detached worktree at origin/main" {
  notes=$'## [Unreleased]\n\n- Sample change.\n\n'
  release_repo
  git worktree add -q --detach "$CASE_ROOT/release-worktree" HEAD
  cd "$CASE_ROOT/release-worktree"
  export CHROME_BIN="$CASE_ROOT/not-installed"

  run release "$next"
  [ "$status" -ne 0 ]
  has 'Run this from main or with a detached HEAD at origin/main'

  git -C "$repo" commit --allow-empty -qm 'Advance main'
  git update-ref refs/remotes/origin/main refs/heads/main
  run release "$next"
  [ "$status" -ne 0 ]
  has 'Run this from main or with a detached HEAD at origin/main'
  [ -z "$(git status --porcelain)" ]

  git switch --quiet --detach refs/remotes/origin/main
  printf '\nUncommitted change.\n' >> README.md
  run release "$next"
  [ "$status" -ne 0 ]
  has 'working tree must be clean'
  git restore -- README.md

  run release "$next"
  [ "$status" -eq 0 ]
  has 'git push origin HEAD:main'
  grep -Fx "VERSION=\"$next\"" cleancodeshit
  [ -z "$(git -C "$repo" status --porcelain)" ]
}

@test "release rejects malformed, equal and older versions" {
  notes=$'## [Unreleased]\n\n- Sample change.\n\n'
  release_repo
  local bad
  for bad in v"$next" 1.2 01.2.3 "$next-rc.1" "$next+build.1"; do
    run release "$bad"
    [ "$status" -ne 0 ]
    has 'stable SemVer in X.Y.Z form'
  done
  for bad in 0.0.0 "$current"; do
    run release "$bad"
    [ "$status" -ne 0 ]
    has "greater than $current"
  done
  [ -z "$(git status --porcelain)" ]
}

@test "release requires notes under Unreleased" {
  notes=''
  release_repo
  run release "$next"
  [ "$status" -ne 0 ]
  has 'needs a non-empty ## [Unreleased]'
  [ -z "$(git status --porcelain)" ]
  notes=$'## [Unreleased]\n\n### Added\n\n<!-- Add release notes here -->\n\n'
  set_release_notes
  git add CHANGELOG.md
  git commit -qm 'Empty Unreleased section'
  run release "$next"
  [ "$status" -ne 0 ]
  has 'needs a non-empty ## [Unreleased]'
  [ -z "$(git status --porcelain)" ]
}

@test "release updates only the expected files and all site version references" {
  notes=$'## [Unreleased]\n\n### Fixed\n\n- Sample change.\n\n'
  release_repo
  printf 'Example %s.\n' "$current" > site/assets/version.txt
  git add site/assets/version.txt
  git commit -qm 'Add a version reference outside the homepage'
  run release "$next"
  [ "$status" -eq 0 ]
  has "git commit -m \"Release $next\""
  has "git push origin v$next"
  expected=$'CHANGELOG.md\nREADME.md\nSECURITY.md\ncleancodeshit\ncleancodeshit.sha256\ndocs/items.md'
  if [[ "$output" == *'Chrome is missing'* ]]; then
    lacks '  docs/sample-terminal.png'
  else
    expected+=$'\ndocs/sample-terminal.png'
  fi
  expected+=$'\npackaging/homebrew/cleancodeshit.rb.in\nsite/assets/version.txt\nsite/index.html'
  [ "$(git diff --name-only)" = "$expected" ]
  [ -z "$(git ls-files --others --exclude-standard)" ]
  [ -z "$(git tag --list)" ]
  grep -Fx "VERSION=\"$next\"" cleancodeshit
  grep -Fx "## [$next] - $(date +%F)" CHANGELOG.md
  grep -Fx "| $next | Supported, latest release |" SECURITY.md
  grep -Fx "| $current | Not supported, superseded by $next |" SECURITY.md
  grep -F "/releases/tag/v$next" README.md
  grep -F "Code Shit $next" docs/items.md
  grep -F "/refs/tags/v$next.tar.gz" packaging/homebrew/cleancodeshit.rb.in
  grep -F "cleancodeshit $next" packaging/homebrew/cleancodeshit.rb.in
  run node tests/check-site.mjs --version-only
  [ "$status" -eq 0 ]
  run node scripts/release-metadata.mjs verify "v$next" "$CASE_ROOT/notes"
  [ "$status" -eq 0 ]
  [ "$(cat "$CASE_ROOT/notes")" = $'### Fixed\n\n- Sample change.' ]
  run node scripts/release-metadata.mjs verify "v$current" "$CASE_ROOT/wrong-notes"
  [ "$status" -ne 0 ]
  has "Tag must equal v$next"
  printf 'Stale v%s.\n' "$current" >> site/assets/version.txt
  run node tests/check-site.mjs --version-only
  [ "$status" -ne 0 ]
  has "version.txt contains v$current"
}

@test "release warns and leaves the screenshot intact when Chrome is missing" {
  notes=$'## [Unreleased]\n\n- Sample change.\n\n'
  release_repo
  export CHROME_BIN="$CASE_ROOT/not-installed"
  run release "$next"
  [ "$status" -eq 0 ]
  has 'Chrome is missing. Skipping docs/sample-terminal.png.'
  git diff --exit-code -- docs/sample-terminal.png
}
