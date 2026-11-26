#!/usr/bin/env bash
set -eu
script_dir=$(CDPATH='' cd -P -- "$(dirname -- "$0")" && pwd -P)
export TMPDIR="$script_dir/.bats-tmp"
mkdir -p -- "$TMPDIR"
if [ "$#" -eq 0 ]; then set -- "$script_dir"; fi
exec "${BASH_UNDER_TEST:-bash}" "$(command -v bats)" "$@"
