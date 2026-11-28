# Publish the Homebrew tap

The tap is live at [bankerneel/homebrew-tap](https://github.com/bankerneel/homebrew-tap) and serves `Formula/cleancodeshit.rb`. Its CI installs, tests and uninstalls the formula on macOS. The tag workflow updates the formula if `HOMEBREW_TAP_TOKEN` is configured. Otherwise, update it manually after the release is published.

1. Follow the [release steps](../../CONTRIBUTING.md#releasing) to update the template and publish the GitHub release.
2. Download the tag archive specified in [the formula template](cleancodeshit.rb.in) and compute its SHA-256 with `shasum -a 256` or `sha256sum`. The archive and the standalone script have different checksums.
3. In `bankerneel/homebrew-tap`, copy the template to `Formula/cleancodeshit.rb`, replace `@TARBALL_SHA256@` with the tag archive checksum, and push the formula update.
4. Install and test the formula with `brew install bankerneel/tap/cleancodeshit` and `brew test bankerneel/tap/cleancodeshit`.

Users can uninstall the formula with `brew uninstall cleancodeshit`. To remove the tap too, run `brew untap bankerneel/tap`.
