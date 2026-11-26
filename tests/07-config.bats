#!/usr/bin/env bats

load test_helper

@test "Docker config parses repository and tag globs and removes exact tag references" {
  export CCS_DOCKER_ENABLED=1
  printf '%s\n' '# sample' '  docker_images=ccs-test/alpha:?,ccs-test/beta' > "$XDG_CONFIG_HOME/cleancodeshit/config"
  run ccs --dry-run --select dk_custom
  [ "$status" -eq 0 ]
  has "ccs-test/alpha:1"
  has "ccs-test/beta:2"
  lacks "ccs-test/keep:3"
  run ccs_confirm yes --select dk_custom
  [ "$status" -eq 0 ]
  grep -Fx $'docker\t--context\tsample\timage\trm\t--\tccs-test/alpha:1' "$CCS_CLEANUP_LOG"
  grep -Fx $'docker\t--context\tsample\timage\trm\t--\tccs-test/beta:2' "$CCS_CLEANUP_LOG"
  ! grep -F $'\trm\t--\tccs-test/keep:3' "$CCS_CLEANUP_LOG"
}

@test "malformed config is ignored and shell substitutions never execute" {
  export CCS_DOCKER_ENABLED=1
  config="$XDG_CONFIG_HOME/cleancodeshit/config"
  printf 'docker_images=$(touch %s/wrong)\n' "$HOME" > "$config"
  run ccs --list
  [ "$status" -eq 0 ]
  has "Ignoring Docker image patterns"
  [ ! -e "$HOME/wrong" ]
  printf '%s\n' 'docker_images=good' 'docker_images=bad' > "$config"
  run ccs --list
  [ "$status" -eq 0 ]
  has "repeated docker_images"
}

@test "Docker environment patterns override the config, including an empty value" {
  export CCS_DOCKER_ENABLED=1
  printf 'docker_images=ccs-test/alpha\n' > "$XDG_CONFIG_HOME/cleancodeshit/config"
  export CLEANCODESHIT_DOCKER_IMAGES=ccs-test/beta
  run ccs --dry-run --select dk_custom
  [ "$status" -eq 0 ]
  has "ccs-test/beta:2"
  lacks "ccs-test/alpha:1"
  export CLEANCODESHIT_DOCKER_IMAGES=
  run ccs --dry-run --select dk_custom
  [ "$status" -eq 2 ]
}

@test "Docker refuses cleanup when the scanned context endpoint changes" {
  export CCS_DOCKER_ENABLED=1 CCS_DOCKER_ROTATE_ENDPOINT=1 CLEANCODESHIT_DOCKER_IMAGES=ccs-test/alpha
  run ccs_confirm yes --select dk_custom
  [ "$status" -eq 1 ]
  ! grep -F $'\timage\trm\t' "$CCS_CLEANUP_LOG"
  grep -F 'selected Docker endpoint changed' "$XDG_STATE_HOME/cleancodeshit"/cleancodeshit-*
}
