#!/usr/bin/env bash
# Shared shim logging. Every tool call is appended as one TAB separated line
# so tests can assert exact arguments without matching shell quoting.
ccs_shim_log() {
  local tool=$1 arg
  shift
  if [ -z "${CCS_CLEANUP_LOG:-}" ]; then
    return 0
  fi
  {
    printf '%s' "$tool"
    for arg in "$@"; do
      printf '\t%s' "$arg"
    done
    printf '\n'
  } >> "$CCS_CLEANUP_LOG"
}
