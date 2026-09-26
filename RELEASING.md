# Releasing

1. Update the package version in `Cargo.toml` and commit the release changes.
2. Tag the commit with the matching version, for example `git tag v0.1.1`.
3. Push the commit and tag: `git push origin main --tags`.

The GitHub Actions release workflow builds the configured platform binaries and installers, creates a GitHub Release, and publishes the Homebrew formula. Tags must use `vMAJOR.MINOR.PATCH` form. Homebrew publishing additionally requires the `aakash-env/homebrew-tap` repository and the `HOMEBREW_TAP_TOKEN` repository secret.