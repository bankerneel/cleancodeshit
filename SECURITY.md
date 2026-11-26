# Security policy

## Supported versions

| Version | Status |
| --- | --- |
| 1.1.0 | Supported, first public release |
| 1.0.0 | Not supported, private build |
| Earlier | Not supported |

Only the latest public release receives fixes.

## Reporting a vulnerability

Please do not open a public issue for a security problem. Send a private report through the repository's security reporting channel with:

- the affected version;
- a description of the issue and its impact;
- steps to reproduce, using fixture paths rather than real data;
- any suggested fix.

You should get an acknowledgement within three business days. We aim to confirm the issue within seven days and ship a fix or a documented mitigation within 30 days. If the report is declined, you will get a short explanation of why.

Please give us a reasonable window to fix the issue before publishing details.

## What we treat as security relevant

- A path that escapes the running user's home directory, through traversal, a symlink, or an environment override.
- A removal that happens without the required confirmation, or a danger item that a preset or `--yes` can select.
- Running as root, or any path that could remove another user's files.
- A change that causes the command to write to a path outside the documented log directory.
- Unintended network access from the command itself.
- Leakage of private paths or log contents into public documentation or sample output.

## What is out of scope

- Vulnerabilities in third party tools that the command merely cleans up after.
- Data loss that follows from a user confirming a danger item, when the confirmation and risk badge worked as documented.
- Rebuild cost for a safe or caution item, such as re-downloading a browser.
- Issues that need root access to exploit, given that the command refuses to run as root.

## Safe testing

Use a disposable fixture home directory in your worktree or a temporary folder. Never point the cleaner at your real home directory while testing a security report. `--list` and `--dry-run` are read only and are the right first check. See [CONTRIBUTING.md](CONTRIBUTING.md) for the full ground rules.
