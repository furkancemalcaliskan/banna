# Banna Repository Instructions

This file is the canonical working contract for coding agents in this
repository. Tool-specific instruction files should point here instead of
maintaining a second copy of these rules.

## Sources Of Truth

Read the files relevant to the task before editing:

- [README.md](README.md) describes the product and supported workflows.
- [Architecture](docs/architecture.md) defines module ownership and dependency
  direction.
- [Contributing](CONTRIBUTING.md) defines the issue, branch, commit, and pull
  request contract.
- [Maintainer workflow](docs/maintainers.md) defines integration, CI, and
  release behavior.
- [Security policy](SECURITY.md) defines private vulnerability reporting.
- [Generated-code licensing](docs/generated-code-licensing.md) defines the
  licensing boundary for generated output and embedded third-party material.

Source code and committed configuration are authoritative for behavior. Update
documentation in the same change when commands, paths, generated output, or
public behavior change. Do not describe planned work as implemented.

## Repository Shape

Banna is intentionally one Cargo package rather than a workspace of internal
crates. Preserve that shape until a demonstrated compile-time, ownership, or
release boundary justifies another package.

- `src/application/` owns interface-independent use cases and validation.
- `src/ports/` owns side-effect contracts.
- `src/adapters/` implements process, prompt, staging, and persistence ports.
- `src/tui/` and `src/headless.rs` adapt human and automated interfaces.
- `src/generator/` owns target-specific generation; `src/generator.rs` owns
  shared orchestration.
- `src/merge/` owns idempotent mutation grouped by destination concern.
- `templates/` and `projects/` are embedded product inputs, not disposable test
  data. Licensing and provenance rules apply to every addition.
- `tests/` owns executable-level integration contracts.

Both binaries must delegate to library entry points. Do not duplicate a use
case in the TUI and CLI. Keep filesystem, subprocess, and prompt behavior behind
ports when a use case needs to be deterministic or testable.

## Before Editing

1. Inspect the relevant source, tests, documentation, current branch, and worktree.
2. Preserve unrelated tracked, untracked, ignored, and stashed work.
3. Start authorized work from the latest `develop` on an issue branch named
   `<type>/<issue>-<short-description>`.
4. Keep the change within the accepted issue and document observable changes.

Supported types are `feat`, `fix`, `refactor`, `test`, `docs`, `ci`, `release`,
and `chore`. Never implement directly on `develop` or `main`.

## Architecture Rules

- Keep application use cases independent of Ratatui, Dialoguer, and Clap.
- Keep terminal rendering and key handling out of generation and merge modules.
- Pass external commands and prompts through the declared ports.
- Treat generation as a filesystem mutation: validate before writing, report
  contextual errors, and remain idempotent where possible.
- Keep target-specific behavior in the matching generator module.
- Keep merge behavior grouped by the file or layer it mutates and protect it
  with repeated-application tests.
- Keep JSON stdout machine-readable; progress and diagnostics belong on stderr.
- Dry-run must not change the selected project, history, project index, or
  external systems.
- Do not add a new internal crate, framework layer, service locator, or generic
  abstraction without a concrete second use and a documented boundary.

## Embedded Content And Security

Never add copied templates, scaffolds, fonts, images, or other embedded content
without recording source, version, license, local scope, and required notices.
Run the provenance and dependency checks described in the release checklist.

Never commit credentials or use a persistent/production project as a test
fixture. Use temporary directories. Do not weaken path validation, dry-run
isolation, command logging redaction, or private vulnerability reporting to
make a check pass.

## Validation

Run the smallest relevant tests while iterating, then the full local gate before
handoff:

```sh
sh scripts/check.sh
```

The gate checks repository policy, formatting, Clippy, all tests, the release
build, binary help contracts, and packaging. Run the dependency gate separately
when `cargo-deny` is installed:

```sh
cargo deny check
```

If a required check cannot run, report the exact command and reason. Do not
claim validation that was not performed.

## Change Authority

Editing and validating local files does not grant authority to commit, push,
open or merge a pull request, change repository settings, tag, publish a crate,
or create a release. Those external mutations require explicit maintainer
authorization.

