#!/usr/bin/env bash
set -eu

scratch=$(CDPATH='' cd -P -- "$1" && pwd -P)
case "${scratch##*/}" in ccs-readme-shot-*) ;; *) exit 1 ;; esac
[ ! -e "$scratch/home" ] || exit 1
export HOME="$scratch/home"
export XDG_CONFIG_HOME="$HOME/.config" XDG_CACHE_HOME="$HOME/.cache" XDG_DATA_HOME="$HOME/.local/share"
export XDG_STATE_HOME="$HOME/.local/state" CODEX_HOME="$HOME/.codex" CLAUDE_CONFIG_DIR="$HOME/.claude"
export CCS_DOCKER_ENABLED=1 CCS_FAKE_UNAME=Linux CCS_REAL_UNAME="$2"
export CCS_DOCKER_ENDPOINT_FILE="$scratch/docker" CCS_CLEANUP_LOG="$scratch/commands"
export CCS_README_SHOT_CLI="$scratch/cleancodeshit"
export TERM=xterm-256color COLORTERM=truecolor COLUMNS=120
shims=$(CDPATH='' cd -P -- "$(dirname -- "$0")/../../tests/shims" && pwd -P)
CCS_REAL_STAT=$(command -p -v stat)
export CCS_REAL_STAT

printf 'unix:///sample-docker.sock\n' > "$CCS_DOCKER_ENDPOINT_FILE"
mkdir -p "$scratch/bin"
cat > "$scratch/bin/df" <<'DF'
#!/usr/bin/env bash
printf '%s\n' 'Filesystem 1024-blocks Used Available Capacity Mounted' 'sample 1048576000 734003200 314572800 70% /sample'
DF
chmod +x "$scratch/bin/df"
export PATH="$scratch/bin:$shims:$PATH"

seed() {
  mkdir -p -- "${1%/*}"
  dd if=/dev/zero of="$1" bs=1024 count="$2" 2>/dev/null
}

seed "$HOME/.npm/_cacache/content-v2/fixture" 1156
seed "$HOME/.npm/_npx/demo/fixture" 6580
seed "$XDG_CACHE_HOME/uv/fixture" 2936
seed "$XDG_CONFIG_HOME/Code/CachedData/fixture" 910
seed "$HOME/.codex/log/codex-tui.log" 1375
seed "$HOME/.codex/sessions/2026/11/04/rollout-sample.jsonl" 2320
seed "$HOME/.claude/debug/abc12345-1234-2345-3456-123456789abc.txt" 400
seed "$HOME/.cache/puppeteer/chrome/fixture" 12364
seed "$HOME/.cache/huggingface/hub/models--sample--model/snapshots/demo/model.bin" 11338

# Both implementations give the CLI a real 120-column terminal.
# shellcheck disable=SC2016
command='stty cols 120; exec /bin/bash "$CCS_README_SHOT_CLI" --dry-run --select safe'
if [ "$CCS_REAL_UNAME" = Darwin ]; then
  script -q /dev/null /bin/bash -c "$command" </dev/null
else
  script -q -e -c "$command" /dev/null </dev/null
fi
