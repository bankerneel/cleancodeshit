#!/usr/bin/env bash
set -euo pipefail

summary() {
  printf '%s\n' "$1" | tee -a "$GITHUB_STEP_SUMMARY"
}

if [ -z "${AUR_SSH_PRIVATE_KEY:-}" ]; then
  summary 'AUR upload skipped because AUR_SSH_PRIVATE_KEY is unavailable.'
  exit 0
fi

version=$1
package_dir=$2
work=$(mktemp -d)
unset SSH_AGENT_PID
trap 'if [ -n "${SSH_AGENT_PID:-}" ]; then ssh-agent -k >/dev/null || :; fi; rm -rf -- "$work"' EXIT
install -m 600 "$(dirname -- "$0")/known_hosts" "$work/known_hosts"
export GIT_SSH_COMMAND="ssh -F /dev/null -o BatchMode=yes -o StrictHostKeyChecking=yes -o HostKeyAlgorithms=ssh-ed25519 -o UserKnownHostsFile=$work/known_hosts -o GlobalKnownHostsFile=/dev/null"
agent=$(ssh-agent -s)
eval "$agent" >/dev/null
printf '%s\n' "$AUR_SSH_PRIVATE_KEY" | ssh-add - >/dev/null
unset AUR_SSH_PRIVATE_KEY

auth_rejected() {
  grep -Eiq 'permission denied \([^)]*publickey|remote: .*(permission denied|authentication (failed|required)|access denied|not authorized|no such user)' "$work/git.log"
}

if ! git clone ssh://aur@aur.archlinux.org/cleancodeshit.git "$work/aur" > "$work/git.log" 2>&1; then
  cat "$work/git.log" >&2
  if auth_rejected; then
    summary 'AUR upload skipped because the AUR rejected SSH authentication. The GitHub release assets are available.'
    exit 0
  fi
  exit 1
fi

current=$work/aur/.SRCINFO
if [ -f "$current" ]; then
  installed_version=$(awk '$1 == "pkgver" && $2 == "=" {print $3; exit}' "$current")
  [ -n "$installed_version" ] || { printf 'AUR .SRCINFO has no pkgver.\n' >&2; exit 1; }
  comparison=$(vercmp "$installed_version" "$version")
  if [ "$comparison" -gt 0 ]; then
    summary "AUR upload skipped because $installed_version is newer than $version."
    exit 0
  fi
  recipe_version=$(sed -nE 's/^pkgver=([0-9]+\.[0-9]+\.[0-9]+)$/\1/p' "$work/aur/PKGBUILD")
  if [ "$recipe_version" != "$installed_version" ]; then
    summary 'AUR upload skipped because the PKGBUILD and .SRCINFO versions disagree.'
    exit 0
  fi
elif [ -e "$work/aur/PKGBUILD" ]; then
  printf 'AUR package is missing .SRCINFO.\n' >&2
  exit 1
fi

cp --remove-destination -- "$package_dir/PKGBUILD" "$package_dir/.SRCINFO" "$work/aur/"
cd "$work/aur"
git add -- PKGBUILD .SRCINFO
if git diff --cached --quiet; then
  summary "AUR already has cleancodeshit $version."
  exit 0
fi
git -c user.name='Neel Banker' -c user.email=neelbanker1@gmail.com commit -m "Update cleancodeshit to $version"
if ! git push --porcelain origin HEAD:refs/heads/master > "$work/git.log" 2>&1; then
  cat "$work/git.log" >&2
  if auth_rejected || grep -Eq '^![[:space:]].*\[(remote rejected|rejected)\]' "$work/git.log"; then
    summary 'AUR upload skipped because the AUR rejected the push. The GitHub release assets are available.'
    exit 0
  fi
  exit 1
fi
summary "Published cleancodeshit $version to the AUR."
