# Items, paths, and sources

This is the reference for every item Clean Code Shit 1.2.0 can clean. Paths are defaults unless an override is named. `<project>`, `<publisher>`, `<model>`, and `YYYY/MM/DD` are placeholders for values from your own system. macOS uses `~/Library/Caches` as its cache root. Linux uses `XDG_CACHE_HOME`, defaulting to `~/.cache`. The scan ignores filesystem targets outside the physical home directory.

Rows marked **danger** need the full word `yes` at the confirmation prompt. Rows marked **caution** are never selected by the `safe` preset. Stale folders are opt-in only and are never in any preset.

## Package managers

| ID | Path | Action | Risk | Official source |
| --- | --- | --- | --- | --- |
| `npm` | `$(npm config get cache)/_cacache`, default `~/.npm/_cacache` | `npm cache clean --force` | safe | [npm-cache](https://docs.npmjs.com/cli/v11/commands/npm-cache) |
| `npx` | `$(npm config get cache)/_npx` | Remove the `_npx` folder | safe | [npx](https://docs.npmjs.com/cli/v11/commands/npx), [npm exec cache paths](https://github.com/npm/cli/blob/latest/workspaces/libnpmexec/README.md#usage) |
| `pnpm` | `${PNPM_HOME:-$XDG_DATA_HOME/pnpm}/store/v*` on Linux, `${PNPM_HOME:-$HOME/Library/pnpm}/store/v*` on macOS | `pnpm store prune` after validating the store selected by the installed pnpm | safe | [pnpm store](https://pnpm.io/cli/store), [store location](https://pnpm.io/settings#store-dir) |
| `yarn` | macOS `~/Library/Caches/Yarn`. Linux `$XDG_CACHE_HOME/yarn`, default `~/.cache/yarn`; `YARN_CACHE_FOLDER` overrides the root | Remove versioned Yarn Classic cache directories beneath the root | safe | [Yarn cache](https://classic.yarnpkg.com/lang/en/docs/cli/cache/) |
| `bun` | `~/.bun/install/cache` | Remove the install cache | safe | [bun pm cache](https://bun.sh/docs/cli/pm#bun-pm-cache) |
| `uv` | `${UV_CACHE_DIR:-$XDG_CACHE_HOME/uv}` | `uv cache prune`. Unused entries only, so the size shown can be larger than the saving | safe | [uv cache location](https://docs.astral.sh/uv/concepts/cache/#cache-directory), [uv cache commands](https://docs.astral.sh/uv/reference/cli/#uv-cache) |
| `pip` | macOS `~/Library/Caches/pip`. Linux `$XDG_CACHE_HOME/pip`, default `~/.cache/pip` | Remove `http`, `http-v2`, `wheels` and `selfcheck` under the default root or `PIP_CACHE_DIR` | safe | [pip cache layout](https://pip.pypa.io/en/stable/topics/caching/) |
| `brew` | `HOMEBREW_CACHE` or `~/Library/Caches/Homebrew` on macOS, `${XDG_CACHE_HOME:-$HOME/.cache}/Homebrew` on Linux | `brew cleanup --prune=all`. Also removes old versions of installed formulae | caution | [Homebrew environment and cleanup](https://docs.brew.sh/Manpage) |

## Build and tool caches

| ID | Path | Action | Risk | Official source |
| --- | --- | --- | --- | --- |
| `xcode` | `~/Library/Developer/Xcode/DerivedData` | Remove DerivedData. Xcode rebuilds it | safe | [Apple: DerivedData](https://developer.apple.com/documentation/xcode/derived-data) |
| `playwright` | macOS `~/Library/Caches/ms-playwright`. Linux `$XDG_CACHE_HOME/ms-playwright` | Remove downloaded browsers; `npx playwright install` brings them back | caution | [Playwright browsers](https://playwright.dev/docs/browsers), [CLI](https://playwright.dev/docs/cli) |
| `electron` | macOS `~/Library/Caches/electron` and `~/Library/Caches/electron-builder`. Linux `$XDG_CACHE_HOME/electron` and `$XDG_CACHE_HOME/electron-builder` | Remove both download caches | safe | [Electron environment variables](https://www.electronjs.org/docs/latest/api/environment-variables#electron_cache), [electron-builder](https://www.electron.build/) |
| `nodegyp` | macOS `~/Library/Caches/node-gyp`. Linux `$XDG_CACHE_HOME/node-gyp` | Remove downloaded Node headers | safe | [node-gyp installation](https://github.com/nodejs/node-gyp#installation), [source](https://github.com/nodejs/node-gyp/blob/main/lib/node-gyp.js) |
| `typescript` | macOS `~/Library/Caches/typescript` and Linux `$XDG_CACHE_HOME/typescript` (default `~/.cache/typescript`) | Remove the automatic type acquisition cache | safe | [TypeScript typings cache source](https://github.com/microsoft/TypeScript/blob/v5.9.3/src/tsserver/nodeServer.ts) |
| `go` | `GOCACHE` or `go-build` under the platform cache root | `go clean -cache` | safe | [Go build cache](https://pkg.go.dev/cmd/go#hdr-Build_and_test_caching), [Go cache roots](https://pkg.go.dev/os#UserCacheDir) |
| `cargo` | `~/.cargo/registry/cache` | Remove crate archives. Sources and installed binaries stay | safe | [Cargo home](https://doc.rust-lang.org/cargo/guide/cargo-home.html) |
| `cocoapods` | macOS `~/Library/Caches/CocoaPods` on macOS only | Remove the CocoaPods cache | safe | [CocoaPods guide](https://guides.cocoapods.org/using/using-cocoapods.html), [config source](https://github.com/CocoaPods/CocoaPods/blob/master/lib/cocoapods/config.rb) |
| `gradle` | `~/.gradle/caches`, or `$GRADLE_USER_HOME/caches` when set | Remove cached dependencies. They re-download on the next build | caution | [Gradle directory layout](https://docs.gradle.org/current/userguide/directory_layout.html) |
| `precommit` | `$PRE_COMMIT_HOME` or `${XDG_CACHE_HOME:-$HOME/.cache}/pre-commit` | Remove the environment directories and their cache index | safe | [pre-commit environment variables](https://pre-commit.com/#environment-variables), [store source](https://github.com/pre-commit/pre-commit/blob/main/pre_commit/store.py) |

## Docker

Docker items go through the Docker CLI rather than filesystem paths. Sizes come from `docker system df` ([docs](https://docs.docker.com/reference/cli/docker/system/df/)). The scan lists containers before images, because an image is only unused once its containers are gone, and volumes last.

| ID | what it targets | Action | Risk | Official source |
| --- | --- | --- | --- | --- |
| `dk_all` | All containers, running ones included | `docker stop`, then `docker rm` the same set of containers. Their data is lost | danger | [docker stop](https://docs.docker.com/reference/cli/docker/container/stop/), [docker rm](https://docs.docker.com/reference/cli/docker/container/rm/) |
| `dk_stopped` | Stopped containers | `docker container prune -f`. Writable layers are lost | danger | [docker container prune](https://docs.docker.com/reference/cli/docker/container/prune/) |
| `dk_custom` | Tagged images whose repository or repository:tag matches a configured pattern | `docker image rm` each matching tag without force | danger | [docker image rm](https://docs.docker.com/reference/cli/docker/image/rm/), configuration below |
| `dk_images` | Unused images | `docker image prune -af`. Local-only images may be lost | danger | [docker image prune](https://docs.docker.com/reference/cli/docker/image/prune/) |
| `dk_build` | Build cache | `docker builder prune -af` | safe | [docker builder prune](https://docs.docker.com/reference/cli/docker/builder/prune/) |
| `dk_net` | Unused custom networks | `docker network prune -f` | safe | [docker network prune](https://docs.docker.com/reference/cli/docker/network/prune/) |
| `dk_volumes` | Unused named and anonymous volumes | `docker volume prune -af`, falling back to `docker volume prune -f`. Databases live here | danger | [docker volume prune](https://docs.docker.com/reference/cli/docker/volume/prune/), [volumes](https://docs.docker.com/engine/storage/volumes/) |

`dk_custom` takes its patterns from `CLEANCODESHIT_DOCKER_IMAGES`, or from the `docker_images=` key in `$XDG_CONFIG_HOME/cleancodeshit/config` (default `~/.config/cleancodeshit/config`). Separate entries with commas or spaces. `*` and `?` are glob wildcards matched against repository names and repository:tag. For example, `example/worker:dev-*` matches those tags only. Config is parsed as data. The environment variable takes precedence over the config file. If neither is set, the item is not offered.

## Editors

| ID | Path | Action | Risk | Official source |
| --- | --- | --- | --- | --- |
| `vscode` | Linux `${XDG_CONFIG_HOME:-$HOME/.config}/Code/logs`, `CachedData`, `Cache`, `CachedExtensionVSIXs`. MacOS `$HOME/Library/Application Support/Code/` with the same subfolders | Remove those cache and log folders only. Settings and extensions stay | safe | [VS Code `userDataPath.ts`](https://github.com/microsoft/vscode/blob/main/src/vs/platform/environment/node/userDataPath.ts), [`environmentService.ts`](https://github.com/microsoft/vscode/blob/main/src/vs/platform/environment/common/environmentService.ts), [cache path list in VS Code source](https://github.com/microsoft/vscode/blob/main/.agents/skills/launch/SKILL.md) |
| `cursor` | macOS `$HOME/Library/Application Support/Cursor/logs`, `CachedData`, `Cache`, `Code Cache`, `GPUCache`, `CachedExtensionVSIXs` | Remove those cache and log folders only | safe | Cursor forum, staff member deanrie: [topic 157419, post 40](https://forum.cursor.com/t/157419/40), [topic 164804, post 5](https://forum.cursor.com/t/164804/5), [topic 153436](https://forum.cursor.com/t/153436) |
| `windsurf` | `Windsurf/{logs,crashpad}` and `Devin/{logs,crashpad}` under `~/Library/Application Support` on macOS or the XDG config root on Linux | Remove logs and crash dumps | safe | [Devin Desktop data paths and migration exclusions](https://docs.devin.ai/desktop/devin-desktop-faq#what-local-file-paths-does-the-application-read-from-and-write-to), [Electron appData](https://www.electronjs.org/docs/latest/api/app#appgetpathname) |

VS Code path overrides `VSCODE_APPDATA` and `VSCODE_PORTABLE` are honoured before the XDG and macOS defaults. Cursor is offered on macOS only in 1.2.0. The scan skips these editor items while the respective application is running. Generic OS temporary directories and editor local history are excluded.

## AI agent logs and history

History items hold prompts, messages, tool output, or session transcripts. Removing them loses that record permanently, so they are **danger** items and require a typed `yes`. Log and debug items are rebuildable and marked **safe**.

| ID | Path | Action | Risk | Official source |
| --- | --- | --- | --- | --- |
| `claude_debug` | `${CLAUDE_CONFIG_DIR:-$HOME/.claude}/debug`; `CLAUDE_CODE_DEBUG_LOGS_DIR` can point to one in-home file with a session UUID or `claude-debug` log name | Remove debug log files | safe | [Claude directory](https://code.claude.com/docs/en/claude-directory.md), [environment variables](https://code.claude.com/docs/en/env-vars.md) |
| `claude_history` | `${CLAUDE_CONFIG_DIR:-$HOME/.claude}/projects/<project>/*.jsonl` | Remove session transcripts | danger | [Claude directory](https://code.claude.com/docs/en/claude-directory.md) |
| `codex_logs` | `${CODEX_HOME:-$HOME/.codex}/log` | Remove CLI log files | safe | [Codex environment variables](https://developers.openai.com/codex/config-file/environment-variables.md), [advanced config](https://developers.openai.com/codex/config-file/config-advanced.md) |
| `codex_history` | `${CODEX_HOME:-$HOME/.codex}/sessions/YYYY/MM/DD/rollout-*.jsonl`, `${CODEX_HOME:-$HOME/.codex}/archived_sessions/rollout-*.jsonl`, and `${CODEX_HOME:-$HOME/.codex}/history.jsonl` | Remove rollout transcripts and history records | danger | [rollout recorder source](https://github.com/openai/codex/blob/main/codex-rs/rollout/src/recorder.rs), [environment variables](https://developers.openai.com/codex/config-file/environment-variables.md), [advanced config](https://developers.openai.com/codex/config-file/config-advanced.md) |
| `gemini_history` | `${GEMINI_CLI_HOME:-$HOME}/.gemini/tmp/<project>/chats/session-*.json` and `session-*.jsonl` | Remove recorded chats | danger | [`storage.ts`](https://github.com/google-gemini/gemini-cli/blob/main/packages/core/src/config/storage.ts), [`chatRecordingService.ts`](https://github.com/google-gemini/gemini-cli/blob/main/packages/core/src/services/chatRecordingService.ts) |
| `copilot_logs` | `${COPILOT_HOME:-$HOME/.copilot}/logs` | Remove process and extension logs. Official docs mark `logs/` safe to delete | safe | [Copilot CLI config directory reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference) |
| `opencode_logs` | `${XDG_DATA_HOME:-$HOME/.local/share}/opencode/log` | Remove log files. They can contain prompts or file contents, so review before sharing them | safe | [OpenCode troubleshooting](https://opencode.ai/docs/troubleshooting/) |

## Browser and model caches

These files are large and slow to fetch again. They are rebuildable, but they are listed apart from safe items and never selected by the `safe` preset.

| ID | Path | Action | Risk | Official source |
| --- | --- | --- | --- | --- |
| `puppeteer` | `${PUPPETEER_CACHE_DIR:-$HOME/.cache/puppeteer}/{chrome,chrome-headless-shell,chromium,firefox}` | Remove downloaded browser binaries. Reinstall with `npx puppeteer browsers install` | caution | [Puppeteer configuration](https://pptr.dev/guides/configuration), [`Cache.ts`](https://github.com/puppeteer/puppeteer/blob/main/packages/browsers/src/Cache.ts) |
| `hf_hub` | `HF_HUB_CACHE`, then legacy `HUGGINGFACE_HUB_CACHE`, then `$HF_HOME/hub`, then `${XDG_CACHE_HOME:-$HOME/.cache}/huggingface/hub` | Remove model repos under `models--<repo>`. They re-download on next use | caution | [Hugging Face local cache](https://huggingface.co/docs/hub/local-cache) |
| `ollama` | `${OLLAMA_MODELS:-$HOME/.ollama/models}` with `manifests/`, `manifests-v2/` and `blobs/`. The resolved path must stay inside HOME | Remove model manifests and blobs | caution | [Ollama FAQ](https://docs.ollama.com/faq), [CLI](https://docs.ollama.com/cli), [disk layout source](https://github.com/ollama/ollama/blob/main/manifest/paths.go) |
| `torch_hub` | `$TORCH_HOME/hub` or `${XDG_CACHE_HOME:-$HOME/.cache}/torch/hub` | Remove downloaded repos and checkpoints | caution | [PyTorch Hub](https://pytorch.org/docs/stable/hub.html) |
| `lm_studio` | `$HOME/.lmstudio/models/<publisher>/<model>/*.gguf` | Remove GGUF model files. Presets and app settings are untouched | caution | [LM Studio import model](https://lmstudio.ai/docs/app/advanced/import-model) |

## Opt-in stale projects

The `stale` item is excluded from all presets. Pass `--in DIR` to add project folders under `DIR`, and `--older-than DAYS` to change the inactivity threshold from 30 days. Choose a directory strictly inside your home directory. For a Git repository, choose its root or a containing directory so the scan can verify its index. Each artifact is listed with its age and size. Selecting `stale` confirms the listed artifacts together. Ambiguous artifact names such as `build`, `dist`, and `target` need evidence that they are generated output.

| ID | Path | Action | Risk | Source |
| --- | --- | --- | --- | --- |
| `stale` | `node_modules`, `.venv`, `venv`, `target`, `.next`, `dist`, `build`, `.turbo`, or `.parcel-cache` under the selected root | Remove the listed artifact directories after checking source age again | caution | [Stale project scanner](../cleancodeshit) |
