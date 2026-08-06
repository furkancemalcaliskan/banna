# Contributing To Banna

Banna welcomes focused bug fixes, tests, documentation improvements, and
discussed features. Contributions from humans and coding agents are reviewed
under the same product, architecture, licensing, and quality requirements.

Please follow the [Code of conduct](CODE_OF_CONDUCT.md). Report suspected
vulnerabilities privately according to [SECURITY.md](SECURITY.md), never in a
public issue or pull request.

## Choose And Discuss Work

Search existing issues before starting. Small documentation corrections may be
proposed directly. Open an issue before substantial behavior changes,
generated-output changes, new dependencies, embedded content, architectural
refactors, or breaking CLI changes so scope and compatibility can be agreed.

An accepted issue should describe the user problem, observable acceptance
criteria, affected targets, compatibility expectations, and required tests.
Being assigned an issue avoids duplicated work but does not guarantee that a
particular implementation will be merged.

## Branch And Commit Contract

`develop` is the default integration branch; `main` is reserved for
release-ready history. Create a branch from the latest `develop`:

```text
<type>/<issue>-<short-description>
```

Supported types are `feat`, `fix`, `refactor`, `test`, `docs`, `ci`, `release`,
and `chore`. Example:

```text
feat/42-export-generation-report
```

Use focused commits with this form:

```text
#42 feat(cli): export a generation report
```

Do not develop directly on `develop` or `main`. Do not mix unrelated cleanup
with the issue change.

## Implementation Expectations

Read [the architecture guide](docs/architecture.md) and, for agent-assisted
work, [AGENTS.md](AGENTS.md). In particular:

- keep TUI and CLI behavior backed by shared application use cases;
- keep target-specific generation in its target module;
- keep external processes, prompts, and persistence behind ports;
- preserve dry-run isolation and JSON stdout contracts;
- make merge behavior idempotent and test repeated application;
- update generated-output fixtures when behavior changes; and
- record provenance and licensing before adding embedded third-party material.

Generated fixtures under `templates/` and `projects/` are product code. A change
to them must include appropriate tests and a review of generated-output and
licensing impact.

## Validation

Run focused tests while developing, then the full gate:

```sh
sh scripts/check.sh
```

When dependency or embedded-content scope changes, also run:

```sh
cargo deny check
```

Explain in the pull request any check that could not run. Never use a
production project or database as a test target.

## Pull Requests

Ordinary pull requests target `develop`, use the repository template, and have
a title in this form:

```text
<type>(<scope>): <description>
```

The body must contain exactly one `Closes #<issue>` matching the issue number
in the branch name. Describe the outcome, exact validation, generated-output or
compatibility impact, licensing impact, and security considerations. Resolve
review conversations and keep the branch current before merge.

Maintainers squash ordinary issue pull requests. A release promotion from
`develop` to `main` is the documented exception and uses a merge commit. See
[the maintainer workflow](docs/maintainers.md).

## Contribution License

By contributing original work, you agree that it is licensed under the Apache
License 2.0 covering Banna. Original generator-template contributions also
carry the generated-output permission in
[`docs/generated-code-licensing.md`](docs/generated-code-licensing.md).

Changes derived from or made directly to the ABP React Native scaffold remain
LGPL-3.0-only. Preserve upstream attribution and license files and document
material changes. Do not contribute code, assets, or templates unless you have
the right to license them on these terms.
