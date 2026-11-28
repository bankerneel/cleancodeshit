# Contributing

Thanks for helping improve Clean Code Shit. The project is a single bash script with documentation around it, so the most useful contributions are careful bug reports, well scoped code changes, and documentation that cites official sources.

## Ground rules

- The command must work with the bash 3.2 that ships with macOS. Avoid bash 4 features such as associative arrays and `${var^^}`.
- Interactive cleanup requires confirmation. Danger items must keep requiring a typed `yes`. Presets exclude danger and stale folders.
- Every filesystem path must stay inside the running user's home directory, with no traversal, no symlink components, and no `.git` directories.
- Never run the cleaner against your real home directory while developing. Use a fixture home in your worktree or a temporary directory.
- Never inspect or publish paths from a real home directory. Documentation examples must use fake data and be labelled as samples.
- Keep probes read-only. Docker items use the selected context, and Ollama can report sizes from a local server.

## Reporting a bug

Open an issue with:

- the version from `cleancodeshit --version`;
- your OS and shell, and whether the terminal was interactive;
- the exact command you ran, with paths redacted;
- the relevant lines from the private run log, also redacted;
- what you expected and what happened.

For a security issue, use the process in [SECURITY.md](SECURITY.md) instead of a public issue.

## Proposing a change

1. Fork or branch from `main`.
2. Keep the change small and focused. One item or one behaviour per pull request is easier to review.
3. Add or update tests for each new probe and cleanup. Run `bash tests/run-tests.sh` with Bats 1.13.0 available.
4. Run the local checks below.
5. Update `README.md`, `docs/items.md`, and `CHANGELOG.md` when behaviour or paths change.

### Local checks

```sh
bash -n cleancodeshit
bash cleancodeshit --version
shellcheck cleancodeshit install.sh
bash tests/run-tests.sh
```

`--list` is read only. Never run a cleaning command to verify a change unless you are pointed at a disposable fixture home.

## Website

`site/` is plain static HTML, CSS, JavaScript, fonts, and images. After the site checks pass, GitHub Actions uploads this directory to the `cleancodeshit` Cloudflare Pages project. Changes to `site/` on `main` deploy to production. Pull requests from this repository with site changes deploy a preview for their branch.

The deployment uses the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` repository secrets. Each upload adds its deployment URL to the Actions job summary.

When changing an item registration, update `CLI_ITEMS` in `site/assets/app.js` in the same pull request. The release command updates the site's release references. Run the site checks with Node.js 22 or newer and Chrome installed.

```sh
node tests/check-site.mjs
```

Set `CHROME_BIN` if Chrome is installed in a nonstandard location. The checks start and stop a loopback server that applies the site's `_headers`. To keep the local server open for manual testing, run `node tests/check-site.mjs --serve` and visit the printed URL. Stop it with Ctrl+C.

## Releasing

Add release notes under `## [Unreleased]` in `CHANGELOG.md` and commit them. Run `scripts/release.sh X.Y.Z` with Node.js 22 or newer from a clean `main` checkout or a clean detached worktree at `origin/main`. It updates the CLI checksum, documentation, packaging, and site. It uses installed Chrome and the system `script` command to refresh the README screenshot, and warns if Chrome is missing.

Review the diff, then run the commands printed by the script.

```sh
git add -u
git commit -m "Release X.Y.Z"
git push origin HEAD:main
git tag vX.Y.Z
git push origin vX.Y.Z
```

The push to `main` deploys the site. The tag workflow publishes the GitHub release after verifying its assets, then updates the Homebrew tap if `HOMEBREW_TAP_TOKEN` is configured.

## Adding an item

Each item needs four things in `cleancodeshit`: an `add` line with a unique ID, group, risk, and label. A `probe_<id>` function that sets `PROBE_BYTES`. A `clean_<id>` function that does the work. And a table row in `docs/items.md` with the exact path and a link to official documentation or official source. If you cannot find an official source for a path, say so in the pull request rather than guessing.

Risk rules:

- `safe` means the tool rebuilds the data on demand.
- `caution` means re-download is slow or large, or old versions are removed.
- `danger` means data can be lost. These items require a typed `yes` and are never in a preset.

## Documentation style

- Write for someone who wants to free space now, not for someone reading a specification.
- Cite official docs or official source for every path. Include the exact path, including environment overrides and platform differences.
- Use plain ASCII punctuation in prose, including hyphens instead of em or en dashes.
- Label sample output as sample data. Do not attribute example data to a real user or a real scan.

## Pull requests

Describe what changed, why, and how you verified it. Mention any risk change explicitly. Keep the diff free of generated artifacts and private paths. A maintainer will review the change, run the checks, and merge when it is ready.
