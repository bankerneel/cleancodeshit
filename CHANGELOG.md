# Changelog

All notable changes to Clean Code Shit are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). This project does not use semantic versioning for its release names.

## [1.2.0] - 2026-11-28

### Changed

- The terminal banner, risk badges, bars, prompts, and cleanup totals use the website palette in truecolor, 256-colour, and basic ANSI terminals. `NO_COLOR` and `--no-color` produce plain output.
- Plain output (`NO_COLOR` or `--no-color`) no longer draws the scanning spinner or progress line.
- CI downloads the pinned ShellCheck release directly and verifies its checksum.

## [1.1.0] - 2026-11-27

First public release. Version 1.0.0 was a private build used for internal testing and was never published.

### Added

- Public release of the `cleancodeshit` command for macOS and Linux, with `cleancodeshit.sha256` and `install.sh` shipped beside it. `bash install.sh` installs to `~/.local/bin`.
- Interactive scan, choose, confirm, clean, and report flow with per item sizes, progress, and a result table showing what was actually freed.
- Non-interactive flags: `--select ITEMS` and `--yes` for scripts, with `--yes` refusing danger items.
- Read-only modes: `--list` for an audit and `--dry-run` for a full preview that changes nothing.
- Opt-in stale project scanning through `--in DIR` and `--older-than DAYS`. Stale folders stay outside every preset.
- `dk_custom` Docker item with repository patterns from `CLEANCODESHIT_DOCKER_IMAGES` or the `docker_images=` key in `$XDG_CONFIG_HOME/cleancodeshit/config`. `*` and `?` work as glob wildcards.
- Editor cache items for VS Code and Cursor, AI agent log and history items for Claude Code, Codex CLI, Gemini CLI, GitHub Copilot CLI, and OpenCode, plus Puppeteer, Hugging Face Hub, Ollama, PyTorch Hub, and LM Studio caches.
- Accessible output options: `--no-color`, `--ascii`, `NO_COLOR=1`, and `CLEANCODESHIT_ASCII=1`.
- A private log for every cleaning run, keeping the 20 newest files.
- Public documentation: this changelog, `README.md`, `docs/items.md` with exact paths and official sources, `CONTRIBUTING.md`, `SECURITY.md`, and the MIT `LICENSE`.
- Brand assets under `assets/brand/`, including the light and dark logo lockups used at the top of the README.

### Changed

- Risk badges are shown on every item: safe, caution, and danger. Presets `safe` and `all` exclude danger items and stale folders.
- Docker cleanup order is fixed so containers are handled before images and volumes are handled last.
- Documentation and source citations were written from official docs and official source only. No real home directory was inspected.

### Security

- Filesystem removals are confined to the running user's home directory. Traversal, symlink components, and `.git` directories are refused.
- The command refuses to run as root.
- Danger items require the full word `yes` typed at the prompt and cannot be selected by a preset or by `--yes`.

## [1.0.0] - private

Internal baseline. Built for local use by the author and a small group of testers. It introduced the scan, choose, confirm, clean, and report loop, the three risk levels, and the package manager, build cache, and Docker items. It was never released publicly and had no public documentation or installer.
