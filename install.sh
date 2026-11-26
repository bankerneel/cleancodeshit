#!/usr/bin/env bash
# Install from the three files published with a Clean Code Shit release.
set -eu

die() { printf 'install.sh: %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" -ne 0 ] || die 'Run this installer as your regular user.'
[ -n "${HOME:-}" ] && [ "$HOME" != / ] || die 'HOME must name your user directory.'

source_dir=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
source_file="$source_dir/cleancodeshit"
checksum_file="$source_dir/cleancodeshit.sha256"
[ -f "$source_file" ] && [ -f "$checksum_file" ] || die 'Put cleancodeshit and cleancodeshit.sha256 beside install.sh.'

if command -v sha256sum >/dev/null 2>&1; then
  sha256() { sha256sum <"$1"; }
elif command -v shasum >/dev/null 2>&1; then
  sha256() { shasum -a 256 <"$1"; }
else
  die 'Install sha256sum or shasum to verify the release.'
fi

expected=$(cat -- "$checksum_file")
if [[ ! "$expected" =~ ^[0-9a-fA-F]{64}[[:blank:]][[:blank:]]cleancodeshit$ ]]; then
  die 'Invalid checksum file.'
fi
expected=${expected%%[[:blank:]]*}
expected=$(printf '%s' "$expected" | LC_ALL=C tr '[:upper:]' '[:lower:]')
actual=$(sha256 "$source_file")
actual=${actual%%[[:blank:]]*}
[ "$actual" = "$expected" ] || die 'Checksum mismatch. Nothing was installed.'
printf 'Verified SHA-256 for %s\n' "$source_file"

home_dir=$(CDPATH='' cd -P -- "$HOME" && pwd -P)
[ "$home_dir" != / ] || die 'HOME must name your user directory.'
destination="$home_dir/.local/bin"
[ ! -L "$home_dir/.local" ] && [ ! -L "$destination" ] || die 'The destination contains a symbolic link.'
mkdir -p -- "$destination"
[ "$(CDPATH='' cd -P -- "$destination" && pwd -P)" = "$destination" ] || die 'The destination changed.'
staged=$(mktemp "$destination/.cleancodeshit.XXXXXX")
trap 'rm -f -- "$staged"' EXIT
trap 'exit 130' INT TERM
install -m 755 -- "$source_file" "$staged"
actual=$(sha256 "$staged")
[ "${actual%%[[:blank:]]*}" = "$expected" ] || die 'The release file changed while copying.'
[ ! -d "$destination/cleancodeshit" ] && [ ! -L "$destination/cleancodeshit" ] || die 'Destination is a symbolic link or an existing directory.'
mv -f -- "$staged" "$destination/cleancodeshit"
printf 'Installed %s\n' "$destination/cleancodeshit"
case ":$PATH:" in
  *":$destination:"*) ;;
  *) printf 'Add %s to your PATH to run cleancodeshit by name.\n' "$destination" ;;
esac
