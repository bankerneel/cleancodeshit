#!/usr/bin/env bats

load test_helper

uv_calls() { awk -F '\t' '$1 == "uv"' "$CCS_CLEANUP_LOG"; }

locked_uv() {
  fixture "$XDG_CACHE_HOME/uv/fixture"
  export CCS_UV_LOCK="$CASE_ROOT/uv.lock" CLEANCODESHIT_UV_LOCK_TIMEOUT=1
  fixture "$CCS_UV_LOCK"
}

@test "an unlocked uv prune finishes without force or a help probe" {
  fixture "$XDG_CACHE_HOME/uv/fixture"
  run ccs --select uv --yes
  [ "$status" -eq 0 ]
  [ "$(uv_calls)" = "$(printf 'uv\t--cache-dir\t%s\tcache\tprune' "$XDG_CACHE_HOME/uv")" ]
  has "may retry with --force after the lock wait limit"
  lacks "Retrying once"
  lacks "uv cache (unused entries) [--force]"
}

@test "locked uv is stopped, then retried once with visible and logged force notices" {
  locked_uv
  run ccs --select uv --yes
  [ "$status" -eq 0 ]
  [ "$(uv_calls)" = "$(printf 'uv\t--cache-dir\t%s\tcache\tprune\nuv\tcache\tprune\t--help\nuv\t--cache-dir\t%s\tcache\tprune\t--force' "$XDG_CACHE_HOME/uv" "$XDG_CACHE_HOME/uv")" ]
  has "the 1-second lock wait limit"
  has "Retrying once with uv cache prune --force."
  has "Force skips the in-use check and can disturb a running uv process."
  result="${output#*> Result}"
  [[ "$result" == *"uv cache (unused entries) [--force]"* ]]
  [[ "$result" == *"uv: retried with uv cache prune --force after the lock wait limit."* ]]
  grep -F 'Retrying once with uv cache prune --force.' "$XDG_STATE_HOME/cleancodeshit"/cleancodeshit-*
  grep -F 'uv: retried with uv cache prune --force after the lock wait limit.' "$XDG_STATE_HOME/cleancodeshit"/cleancodeshit-*
}

@test "force flag and environment use a single forced uv attempt from the start" {
  locked_uv
  expected=$(printf 'uv\tcache\tprune\t--help\nuv\t--cache-dir\t%s\tcache\tprune\t--force' "$XDG_CACHE_HOME/uv")
  run ccs --select uv --yes --force-uv
  [ "$status" -eq 0 ]
  [ "$(uv_calls)" = "$expected" ]
  has "Using uv cache prune --force from the start."
  has "uv: started with uv cache prune --force."
  has "uv cache (unused entries) [--force]"
  lacks "Retrying once"
  : >"$CCS_CLEANUP_LOG"
  export CLEANCODESHIT_UV_FORCE=1
  run ccs --select uv --yes
  [ "$status" -eq 0 ]
  [ "$(uv_calls)" = "$expected" ]
  has "Using uv cache prune --force from the start."
  has "Force skips the in-use check and can disturb a running uv process."
}

@test "older locked uv fails within the limit and the following item still runs" {
  locked_uv
  fixture "$XDG_CACHE_HOME/go-build/fixture"
  export CCS_UV_NO_FORCE=1
  start=$SECONDS
  run ccs --select 'uv go' --yes
  [ "$status" -eq 1 ]
  [ "$((SECONDS - start))" -lt 12 ]
  [ "$(uv_calls)" = "$(printf 'uv\t--cache-dir\t%s\tcache\tprune\nuv\tcache\tprune\t--help' "$XDG_CACHE_HOME/uv")" ]
  has "Close other uv processes, or update uv, then run again."
  has "OK Go build cache"
  lacks "uv cache (unused entries) [--force]"
  grep -Fx $'go\tclean\t-cache' "$CCS_CLEANUP_LOG"
}

@test "uv returning its own nonzero status does not trigger a forced retry" {
  fixture "$XDG_CACHE_HOME/uv/fixture"
  export CCS_UV_PRUNE_STATUS=124
  run ccs --select uv --yes
  [ "$status" -eq 1 ]
  [ "$(uv_calls)" = "$(printf 'uv\t--cache-dir\t%s\tcache\tprune' "$XDG_CACHE_HOME/uv")" ]
  has "failed (exit 124)"
  lacks "Retrying once"
}
