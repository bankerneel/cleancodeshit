# Publish the Homebrew tap

The tap is live at [bankerneel/homebrew-tap](https://github.com/bankerneel/homebrew-tap) and serves `Formula/cleancodeshit.rb`. Its CI installs, tests and uninstalls the formula on macOS. For each new release, update `url`, `sha256` and the version in the `test` block there, using the steps below.

1. After the release commit is approved and available publicly, tag it `v1.2.0` and publish that tag. Upload `cleancodeshit`, `cleancodeshit.sha256`, and `install.sh` as the three release assets.
2. Download the tag archive specified in [the formula template](cleancodeshit.rb.in) and compute its SHA-256 with `shasum -a 256` or `sha256sum`. The archive and the standalone script have different checksums.
3. Create the `bankerneel/homebrew-tap` repository. Copy the template to `Formula/cleancodeshit.rb` and replace `@TARBALL_SHA256@` with the tag archive checksum.
4. Install and test the formula with `brew install bankerneel/tap/cleancodeshit` and `brew test bankerneel/tap/cleancodeshit` after publishing the tap.

Users can uninstall the formula with `brew uninstall cleancodeshit`. To remove the tap too, run `brew untap bankerneel/tap`.
