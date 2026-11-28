# Arch Linux packaging

`PKGBUILD.in` defines the `cleancodeshit` package. It installs the command, license, README, and item reference. The release job fills in `pkgver` and the SHA-256 of the exact GitHub tag archive, then runs `makepkg --printsrcinfo`. GitHub releases attach `PKGBUILD` and `SRCINFO`. GitHub strips leading periods from asset filenames, so rename downloaded `SRCINFO` to `.SRCINFO` if you need it locally.

The runtime depends on Bash, coreutils (`du`, `df`, `stat`, and file operations), findutils (`find`), gawk (`awk`), grep and sed (text processing), procps-ng (`ps`), and ncurses (`tput`). Git is optional and lets the stale scan inspect tracked files. The other optional tools enable their respective audit or cleanup features.

The Arch CI job renders the same template against a `git archive` of the checkout. It uses a local archive with the expected filename and verifies the package with `makepkg`, `namcap`, and an install, CLI smoke test, and uninstall through `pacman`.

## One-time AUR setup

1. When registration opens, create the maintainer account on the [AUR](https://aur.archlinux.org/). Add the public half of the dedicated deploy key to that account. Follow the [Arch package submission guidelines](https://wiki.archlinux.org/title/AUR_submission_guidelines).
2. Store the corresponding private key as the `AUR_SSH_PRIVATE_KEY` GitHub Actions repository secret. The job loads it through `ssh-agent` and authenticates the server using [`known_hosts`](known_hosts), pinned to the Ed25519 fingerprint published by the [AUR](https://aur.archlinux.org/).
3. Once the account and key are ready, the AUR job pushes the recipe and `.SRCINFO` after each tagged release. The first accepted push creates the package. If the key or account is unavailable, the job records a skip and Arch users can still build from the GitHub `PKGBUILD` asset on new releases.
