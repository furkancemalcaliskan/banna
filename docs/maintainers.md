# Maintainer Workflow

This document defines Banna's repository integration and release contract.
GitHub repository settings should enforce the same rules.

## Branch Model

```text
issue -> issue branch -> pull request -> develop
      -> promotion pull request -> main -> stable tag -> release
```

`develop` is the default and integration branch. Create work from the latest
`develop` using `<type>/<issue>-<short-description>`. Ordinary pull requests
target `develop` and use squash merge. Their title is
`<type>(<scope>): <description>` and the body contains exactly one
`Closes #<issue>` matching the branch.

`main` contains release-ready history only. Promote `develop` to `main` through
a reviewed pull request titled `release: promote banna vX.Y.Z to main`. Use a
merge commit for this promotion so the release boundary remains visible.
Dependabot pull requests may keep their generated branch and title, but target
`develop` and require the same checks and review.

Recommended GitHub settings:

- set `develop` as the default branch;
- protect `develop` and `main` from direct pushes and force pushes;
- require one approving review and resolved conversations;
- require `repository-policy`, `quality`, `dependency-policy`, and `msrv`;
- require branches to be current before merge;
- allow squash merge for issue pull requests and merge commits for promotion;
- enable private vulnerability reporting and immutable releases.

Repository settings are not versioned by these files. Confirm them after the
repository is created or transferred.

## Validation Contract

| Workflow | Trigger | Responsibility | Side effect |
| --- | --- | --- | --- |
| `repository-policy` | PR to `develop` or `main` | Documentation links and PR metadata | None |
| `CI` | PR/push to `develop` or `main` | Format, lint, tests, package, dependencies, MSRV | None |
| `Release` | stable `vMAJOR.MINOR.PATCH` tag | Verify, build, checksum, publish GitHub Release | GitHub Release |

Pull-request workflows have read-only repository permission. Only the release
publication job receives `contents: write`. CI must never publish a crate,
release, deployment, or generated project.

Run the local aggregate gate with:

```sh
sh scripts/check.sh
```

Run repository metadata tests directly with:

```sh
sh scripts/repository-policy.sh
```

To reproduce pull-request policy validation using a saved GitHub event:

```sh
sh scripts/repository-policy.sh --event path/to/event.json
```

## Release Contract

Banna releases platform archives containing `banna`, `banna-cli`, the README,
license, third-party notices, and applicable license texts. Every archive has a
SHA-256 checksum. A GitHub release does not automatically publish to crates.io.

Before promotion, update `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and release
documentation together. The changelog must contain a dated heading in the form
`## [X.Y.Z] - YYYY-MM-DD`. Complete [the release checklist](release-checklist.md)
and run all quality and dependency gates on the intended commit.

After `develop` is promoted, create a signed annotated tag from the verified
`main` commit:

```sh
git tag -s vX.Y.Z -m "Banna vX.Y.Z"
git tag -v vX.Y.Z
git push origin vX.Y.Z
```

The tag is the GitHub publication boundary. Never move or recreate a published
tag. Correct released defects with a new patch release. Publish to crates.io
only as a separate, explicit maintainer action after verifying the GitHub
artifacts:

```sh
cargo publish --locked
```

Local edits, successful validation, or preparation of release files do not
authorize a commit, push, merge, tag, release, or registry publication.

