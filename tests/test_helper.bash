#!/usr/bin/env bash
# shellcheck disable=SC2329
# Functions in this file are invoked by Bats and its test files.

setup() {
  TESTS_DIR=$(CDPATH='' cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
  PRODUCT="${TESTS_DIR%/*}/cleancodeshit"
  CASE_ROOT=$(CDPATH='' cd -P -- "${BATS_TEST_TMPDIR:?}" && pwd -P)
  case "$CASE_ROOT" in "$TESTS_DIR"/.bats-tmp/*) ;; *)
    printf 'Bats must run through tests/run-tests.sh.\n' >&2
    return 1 ;;
  esac
  export CCS_REAL_UNAME CCS_REAL_STAT CCS_REAL_PS
  CCS_REAL_UNAME=$(uname -s)
  CCS_REAL_STAT=$(command -v stat)
  CCS_REAL_PS=$(command -v ps)

  unset BASH_ENV CDPATH VSCODE_APPDATA VSCODE_PORTABLE CLAUDE_CODE_DEBUG_LOGS_DIR \
    CLAUDE_CONFIG_DIR CODEX_HOME GEMINI_CLI_HOME COPILOT_HOME PUPPETEER_CACHE_DIR \
    HF_HOME HF_HUB_CACHE HUGGINGFACE_HUB_CACHE TORCH_HOME OLLAMA_MODELS OLLAMA_HOST \
    DOCKER_HOST DOCKER_CONTEXT DOCKER_CONFIG CLEANCODESHIT_DOCKER_IMAGES GOCACHE \
    npm_config_cache NPM_CONFIG_CACHE PNPM_HOME YARN_CACHE_FOLDER UV_CACHE_DIR \
    PIP_CACHE_DIR HOMEBREW_CACHE PRE_COMMIT_HOME GRADLE_USER_HOME CARGO_HOME
  unset CCS_INTERRUPT_PIDS CCS_DOCKER_ROTATE_ENDPOINT CCS_FAKE_MOUNT GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR
  unset CLEANCODESHIT_UV_FORCE CLEANCODESHIT_UV_LOCK_TIMEOUT CCS_UV_LOCK CCS_UV_NO_FORCE CCS_UV_PRUNE_STATUS
  HOME="$CASE_ROOT/home"
  XDG_CACHE_HOME="$HOME/.cache"
  XDG_CONFIG_HOME="$HOME/.config"
  XDG_DATA_HOME="$HOME/.local/share"
  XDG_STATE_HOME="$HOME/.local/state"
  mkdir -p -- "$HOME" "$XDG_CONFIG_HOME/cleancodeshit"
  CCS_CLEANUP_LOG="$CASE_ROOT/shim.log"
  CCS_PS_OUTPUT_FILE="$CASE_ROOT/ps"
  CCS_OLLAMA_LIST_FILE="$CASE_ROOT/models"
  CCS_DOCKER_ENDPOINT_FILE="$CASE_ROOT/docker-endpoint"
  CCS_FAKE_UNAME=Linux
  CCS_DOCKER_ENABLED=0
  : > "$CCS_CLEANUP_LOG"
  : > "$CCS_PS_OUTPUT_FILE"
  printf '%s\n' 'NAME ID SIZE MODIFIED' 'demo:latest abc123 1.2 GB 2 days ago' > "$CCS_OLLAMA_LIST_FILE"
  printf '%s\n' 'unix:///sample-docker.sock' > "$CCS_DOCKER_ENDPOINT_FILE"
  export HOME XDG_CACHE_HOME XDG_CONFIG_HOME XDG_DATA_HOME XDG_STATE_HOME
  export CCS_CLEANUP_LOG CCS_PS_OUTPUT_FILE CCS_OLLAMA_LIST_FILE CCS_DOCKER_ENDPOINT_FILE
  export CCS_FAKE_UNAME CCS_DOCKER_ENABLED
  export PATH="$TESTS_DIR/shims:$PATH" NO_COLOR=1 TERM=dumb CLEANCODESHIT_ASCII=1
  assert_fake_home
}

assert_fake_home() {
  local base actual
  [ -n "${HOME:-}" ] && [ -n "${BATS_TEST_TMPDIR:-}" ] || return 1
  base=$(CDPATH='' cd -P -- "$BATS_TEST_TMPDIR" 2>/dev/null && pwd -P) || return 1
  actual=$(CDPATH='' cd -P -- "$HOME" 2>/dev/null && pwd -P) || return 1
  case "$actual" in "$base"/*) ;; *) printf 'Refusing to run outside the fake HOME.\n' >&2; return 1 ;; esac
}

ccs() {
  assert_fake_home || return 1
  "${BASH_UNDER_TEST:-bash}" "$PRODUCT" "$@"
}

# Substitute only the stdin TTY test so small scripted replies can exercise the
# real interactive selection and confirmation code on Git Bash as well as Unix.
ccs_confirm() {
  local answer="$1"
  shift
  assert_fake_home || return 1
  # The Bash child expands these arguments and parameters.
  # shellcheck disable=SC2016
  printf '%s\n' "$answer" | "${BASH_UNDER_TEST:-bash}" -c '
    function [ {
      if [[ "${1:-}" = -t && "${2:-}" = 0 ]]; then return 0; fi
      builtin [ "$@"
    }
    product=$1; shift
    . "$product"
  ' "$PRODUCT" "$PRODUCT" "$@"
}

fixture() {
  mkdir -p -- "${1%/*}"
  printf 'sample only\n' > "$1"
}

old() { touch -t 200001010000 -- "$1"; }

make_project() {
  PROJECT_ROOT="$HOME/work/$1"
  fixture "$PROJECT_ROOT/package.json"
  fixture "$PROJECT_ROOT/src/main.js"
  fixture "$PROJECT_ROOT/node_modules/demo/index.js"
  if [ "${2:-old}" = old ]; then
    old "$PROJECT_ROOT/package.json"
    old "$PROJECT_ROOT/src/main.js"
  fi
}

snapshot() { find "$HOME" -print | LC_ALL=C sort; }
has() { [[ "$output" == *"$1"* ]]; }
lacks() { [[ "$output" != *"$1"* ]]; }
plan_only() { output="${output#*> Plan}"; }

link_dir() {
  local target="$1" link="$2"
  case "$CCS_REAL_UNAME" in
    MINGW*|MSYS*)
      node -e 'require("node:fs").symlinkSync(process.argv[1], process.argv[2], "junction")' \
        "$(cygpath -aw "$target")" "$(cygpath -aw "$link")" || return 1 ;;
    *) ln -s -- "$target" "$link" || return 1 ;;
  esac
  [ -L "$link" ]
}
