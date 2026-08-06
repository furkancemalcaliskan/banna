# Release checklist

Use this checklist for the first public release and subsequent tagged releases.

## Current first-release gates

- [x] Record the maintainer's original-authorship statement and unrestricted
      output permission for all Banna generator templates.
- [x] Record the public ABP Framework rel-8.3 upstream, LGPL-3.0-only scope,
      local modifications, and notices for the React Native derivative.
- [x] Initialize local Git history with `develop` as the working integration
      branch and `main` as the release branch.
- [ ] Create the GitHub repository as `furkancemalcaliskan/banna` and configure
      the local `origin`. Cargo metadata already points to the intended public
      URL.
- [ ] Set `develop` as the GitHub default and apply the protections and required
      checks in [the maintainer workflow](maintainers.md).
- [x] Replace the unverified non-Banna mobile artwork while preserving Expo
      dimensions/usages, and record its reproducible Banna source.

## Required before publishing

- [x] Resolve every open item in the
      [embedded-content provenance audit](provenance-audit.md).
- [x] Add `THIRD_PARTY_NOTICES.md` and embed the LGPL notice/license with the
      generated React Native scaffold.
- [ ] Confirm the repository URL, homepage, authors, description, and license in
      `Cargo.toml`.
- [x] Confirm `README.md`, `LICENSE`, `CONTRIBUTING.md`, `SECURITY.md`,
      `SUPPORT.md`, and `CODE_OF_CONDUCT.md` are appropriate for the public
      repository.
- [x] Move the `Unreleased` changelog entries into the versioned release
      section and add the release date.
- [x] Search tracked and packaged files for old `hegira`, `hegira-rs`, and
      `dejkoveci` branding.
- [x] Review the complete `cargo package --list` output for secrets, local
      files, build output, and unintended assets.
- [ ] Run formatting, lint, unit/integration tests, and a release build on a
      clean checkout.
- [ ] Smoke-test `banna --help`, the TUI, and `banna-cli --help`.
- [ ] Smoke-test one representative generation per supported UI target in a
      disposable project and review the generated diff.
- [x] Confirm dry-run leaves the source project, history, and project index
      unchanged.
- [x] Review dependency licenses and known advisories using the selected release
      gate (`cargo deny check`; policy is documented in
      [dependency-policy.md](dependency-policy.md)).
- [x] Decide whether version `0.1.0` accurately communicates the public API and
      compatibility level.
- [x] Write release notes that describe supported targets, limitations, and
      migration from any private/pre-release build.

## Tag and artifact checks

- [ ] Create the release from a reviewed commit with passing CI.
- [ ] Promote the release commit from `develop` to `main` through the required
      release pull request.
- [ ] Build artifacts from the tag, not from an uncommitted working tree.
- [x] Configure checksum and executable-name verification for every distributed
      platform.
- [x] Confirm both `banna` and `banna-cli` are present in binary distributions.
- [x] Install the packaged crate or artifact into a clean environment and run
      its help/version commands.
- [ ] Publish the tag and release notes only after artifact verification.

## After publishing

- [ ] Verify public repository links and installation instructions.
- [ ] Open issues for deferred, non-blocking technical debt.
- [ ] Record the released version and date in the changelog or release notes.
