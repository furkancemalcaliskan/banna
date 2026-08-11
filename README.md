<p align="center">
  <img src="assets/brand/banna.png" width="280" alt="Banna logo">
</p>

<h1 align="center">Banna</h1>

<p align="center"><strong>Shape the model. Generate the layers.</strong></p>

<p align="center">
  A terminal-first code generator for existing ABP applications.
</p>

Banna turns an entity definition into the repetitive domain, application,
Entity Framework Core, and UI code an ABP solution needs. It combines an
interactive terminal interface with a deterministic headless CLI, previews
changes safely, and merges registrations into an existing project.

Banna is deliberately smaller than an application framework. It does not own
your runtime architecture or replace ABP: it automates a focused development
workflow while leaving the generated source in your repository for review and
continued ownership.

> **Independence and trademarks:** Banna is an independent open-source project.
> It is not affiliated with, sponsored by, or officially endorsed by Volosoft
> or the ABP Framework project. The ABP name and related marks belong to their
> respective owners.

> **Project status:** Banna is preparing its first public release. Generated
> output and CLI contracts may still evolve before `1.0.0`. Use version control,
> start with `--dry-run`, and review every generated diff.

## Capabilities

- Generate entities, repositories, managers, application services, DTOs,
  mappings, permissions, and EF Core integration.
- Generate Razor Pages, Vue components, complete standalone Angular CRUD
  features with typed REST proxies, and an optional React Native application
  base when the corresponding host is present.
- Use the same application workflows through the `banna` TUI and `banna-cli`.
- Preview generation and configuration in an isolated workspace with
  `--dry-run`.
- Keep existing registrations untouched with `--no-merge`.
- Maintain a local project index and per-project generation history.
- Inspect, configure, regenerate, and migrate from scripts or CI with JSON
  output that keeps diagnostics on stderr.

## Quick Start

Banna requires Rust 1.97.1 and, for real generation, an existing ABP solution and
its matching .NET SDK. When web generation needs npm, Banna uses the system npm
or downloads and verifies a private Node.js 24 LTS distribution. It also keeps
the latest available ABP CLI in its private tool cache; no `sudo` or administrator
prompt is used. See [Getting started](docs/getting-started.md) for cache paths,
the `BANNA_TOOLCHAIN_DIR` override, dependency resolution, and failure
diagnostics.

Build and run from source:

```sh
cargo build --release --locked --bins
./target/release/banna
```

For tagged releases, download the binary bundle matching your machine:

| Operating system | Architecture | Release target |
| --- | --- | --- |
| Linux | x86-64 | `x86_64-unknown-linux-gnu` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` |
| macOS | Intel | `x86_64-apple-darwin` |
| macOS | Apple Silicon | `aarch64-apple-darwin` |
| Windows | x86-64 | `x86_64-pc-windows-msvc` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` |

Verify the archive with its `.sha256` file or the release `SHA256SUMS`, extract
it, and place both executables in a directory on `PATH`. GitHub's automatic
“Source code” links are not Banna binary distributions.

Inspect the non-interactive interface:

```sh
./target/release/banna-cli --help
```

Preview an entity without changing the source project, project index, history,
or external systems:

```sh
./target/release/banna-cli entity generate \
  --project /path/to/project \
  --domain Acme.Billing \
  --namespace Acme.Billing.Invoices \
  --entity Invoice \
  --fields examples/invoice-fields.json \
  --ui razor \
  --dry-run \
  --non-interactive
```

Remove `--dry-run` only after reviewing the preview. Add `--format json` for
machine-readable completion output, `--no-merge` to avoid registration edits,
or `--run-migration` to run EF Core migration commands after generation.

See the step-by-step [User guide](docs/user-guide.md) for the TUI, field JSON,
all UI targets, themes, localization, regeneration, automation, and
troubleshooting. See [Getting started](docs/getting-started.md) for repository
setup and toolchain details.

## Headless Workflows

Register and inspect projects without opening the TUI:

```sh
banna-cli project add --path /path/to/project --name Demo
banna-cli project list --format json
banna-cli project inspect --path /path/to/project --format json
banna-cli project configure --path /path/to/project --bootstrap-override nord
banna-cli project remove Demo
```

Inspect generation history or regenerate from a saved definition:

```sh
banna-cli history list --project /path/to/project
banna-cli entity regenerate \
  --project /path/to/project \
  --entity Invoice \
  --non-interactive
banna-cli history remove --project /path/to/project --entity Invoice
```

Removing a history entry does not delete generated files. Set
`BANNA_CONFIG_DIR` to isolate the project index in CI and tests.

## Architecture

Banna is one Cargo package with a library boundary and two thin binaries. The
TUI and CLI translate input into shared application use cases. Those use cases
coordinate target generators, idempotent merge operations, and explicit ports
for processes, history, and project-index persistence.

| Surface | Responsibility |
| --- | --- |
| `src/application/` | Interface-independent use cases, validation, and reports |
| `src/ports/`, `src/adapters/` | Side-effect contracts and system implementations |
| `src/tui/`, `src/headless.rs` | Interactive and automated delivery interfaces |
| `src/generator/` | Target-specific generated-output assembly |
| `src/merge/` | Idempotent edits to existing application files |
| `templates/`, `projects/` | Compile-time embedded generator inputs |

The package intentionally remains a small modular monolith; internal crates are
not an architectural goal. See [Architecture](docs/architecture.md) for module
ownership, dependency direction, persistence, dry-run, and licensing
boundaries.

## Documentation

- [User guide](docs/user-guide.md)
- [Getting started](docs/getting-started.md)
- [Architecture](docs/architecture.md)
- [Contribution policy](CONTRIBUTING.md)
- [Maintainer workflow](docs/maintainers.md)
- [Security policy](SECURITY.md)
- [Support](SUPPORT.md)
- [Code of conduct](CODE_OF_CONDUCT.md)
- [Generated-code licensing](docs/generated-code-licensing.md)
- [Dependency policy](docs/dependency-policy.md)
- [Release process](docs/releasing.md)
- [Changelog](CHANGELOG.md)

## Development And Governance

Run the local quality gate with:

```sh
sh scripts/check.sh
```

`develop` is the default integration branch. Create issue branches from
`develop`, open ordinary pull requests back to `develop`, and promote a
verified release from `develop` to `main` through a pull request before tagging.
The repository policy workflow validates this contract automatically.

Human and agent contributors follow the same architecture and validation
rules. Humans should begin with [CONTRIBUTING.md](CONTRIBUTING.md); coding agents
must also follow [AGENTS.md](AGENTS.md).

## Licensing

Banna's original Rust code, documentation, and project-specific content are
available under the [Apache License 2.0](LICENSE), except for identified
third-party material. Original Banna template fragments carry an additional
permission allowing generated output to be used under the recipient's chosen
terms, including proprietary terms and without attribution; see
[Generated-code licensing](docs/generated-code-licensing.md). That permission
does not relicense third-party material.

The embedded React Native scaffold is a modified derivative of the ABP
Framework 8.3 React Native template and remains LGPL-3.0-only. Its exact scope,
source, modifications, and notices are recorded in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). Distributed artifacts include
the applicable LGPLv3 and GPLv3 license texts.
