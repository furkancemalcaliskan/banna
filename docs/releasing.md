# Releasing Banna

Banna releases are built from signed, stable SemVer tags on `main`. The GitHub
Actions workflow produces native archives for Linux x86-64, macOS Apple
Silicon, and Windows x86-64. Every archive contains `banna`, `banna-cli`, the
README, repository license, third-party notices, and applicable LGPLv3/GPLv3
texts. Every archive has a SHA-256 checksum.

## Prepare And Promote

Complete [the release checklist](release-checklist.md) on `develop`. Update the
version in `Cargo.toml` and `Cargo.lock`, move `Unreleased` changelog entries to
a dated `## [X.Y.Z] - YYYY-MM-DD` section, and prepare release notes. Run:

```sh
sh scripts/check.sh
cargo deny check
sh scripts/release-policy.sh vX.Y.Z
sha256sum --check assets/brand/SHA256SUMS
```

Promote the verified `develop` commit to `main` through a pull request titled:

```text
release: promote banna vX.Y.Z to main
```

Do not tag an issue branch or an unreviewed commit.

## Publish GitHub Artifacts

From the verified `main` release commit, create and verify a signed annotated
tag:

```sh
git tag -s vX.Y.Z -m "Banna vX.Y.Z"
git tag -v vX.Y.Z
git push origin vX.Y.Z
```

The tag starts `.github/workflows/release.yml`. The workflow repeats repository,
package, changelog, dependency, provenance, MSRV, test, and build validation. It
creates the GitHub Release only after all platform archives succeed. Verify the
uploaded checksums and generated notes before announcing the release.

Enable immutable GitHub Releases. Never move or recreate a published tag; make
a new patch release to correct a release.

## Publish The Crate

GitHub release automation intentionally does not publish to crates.io. After
verifying GitHub artifacts and package contents, publish as a separate explicit
maintainer action:

```sh
cargo package --locked
cargo publish --locked
```

Confirm the crates.io package exposes both binary targets and includes the
README, third-party notices, embedded-content licenses, templates, and project
inputs required at runtime.

