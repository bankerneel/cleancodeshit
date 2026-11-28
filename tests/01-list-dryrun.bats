#!/usr/bin/env bats

load test_helper

# Exercise the real styling and banner with predictable capabilities on both CI OSes.
ccs_color_tty() {
  assert_fake_home || return 1
  # shellcheck disable=SC2016
  "${BASH_UNDER_TEST:-bash}" -c '
    function [ {
      if [[ "${1:-}" = -t && "${2:-}" = 1 ]]; then return 0; fi
      builtin [ "$@"
    }
    tput() {
      case "$1" in
        cols) echo 112 ;;
        colors) echo 8 ;;
      esac
    }
    product=$1; shift
    . "$product"
  ' "$PRODUCT" "$PRODUCT" "$@"
}

assert_brand_sgr() {
  local esc=$'\033' code
  for code in $(printf '%s\n' "$output" | grep -oE "${esc}\[[0-9;]+m" | tr -d "${esc}[m" | sort -u); do
    case "$code" in
      0|1|2\;37|30\;43|30\;47|31|33|37) ;;
      38\;5\;167|38\;5\;221|38\;5\;245|38\;5\;255|38\;5\;233\;48\;5\;221|38\;5\;233\;48\;5\;255) ;;
      38\;2\;142\;145\;155|38\;2\;229\;56\;59|38\;2\;243\;245\;248|38\;2\;255\;200\;61) ;;
      38\;2\;17\;18\;22\;48\;2\;243\;245\;248|38\;2\;17\;18\;22\;48\;2\;255\;200\;61) ;;
      *) printf 'Unexpected ANSI SGR %s\n' "$code"; return 1 ;;
    esac
  done
}

@test "terminal output uses only brand escapes and respects both plain output options" {
  fixture "$HOME/.bun/install/cache/package"
  fixture "$HOME/.cache/puppeteer/chrome/fixture"
  fixture "$HOME/.codex/sessions/2026/11/27/rollout-fixture.jsonl"
  unset NO_COLOR CLEANCODESHIT_ASCII
  export TERM=xterm-256color COLORTERM
  local esc=$'\033'

  for COLORTERM in truecolor 24bit; do
    run ccs_color_tty --list
    [ "$status" -eq 0 ]
    has "${esc}[38;2;243;245;248m"
    has "${esc}[38;2;255;200;61m"
    has "${esc}[38;2;142;145;155m"
    has "${esc}[38;2;229;56;59mdanger"
    assert_brand_sgr
  done

  unset COLORTERM
  run ccs_color_tty --list
  [ "$status" -eq 0 ]
  has "${esc}[38;5;255m"
  has "${esc}[38;5;221m"
  has "${esc}[38;5;245m"
  has "${esc}[38;5;167mdanger"
  assert_brand_sgr

  export TERM=xterm
  run ccs_color_tty --list
  [ "$status" -eq 0 ]
  has "${esc}[37m"
  has "${esc}[33m"
  has "${esc}[31mdanger"
  assert_brand_sgr

  export COLORTERM=truecolor
  run ccs_color_tty --list --no-color
  [ "$status" -eq 0 ]
  has "danger"
  lacks "$esc"
  export NO_COLOR=1
  run ccs_color_tty --list
  [ "$status" -eq 0 ]
  has "danger"
  lacks "$esc"
}

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
