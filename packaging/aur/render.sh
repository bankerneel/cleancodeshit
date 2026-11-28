#!/usr/bin/env bash
set -eu

if [ "$#" -ne 2 ] || [[ ! "$1" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  printf 'Usage: %s X.Y.Z TAG_ARCHIVE\n' "$0" >&2
  exit 1
fi

digest=$(sha256sum < "$2")
sed -e "s/@PKGVER@/$1/g" -e "s/@SHA256@/${digest%% *}/g" "$(dirname -- "$0")/PKGBUILD.in"
