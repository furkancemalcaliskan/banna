# Getting Started

This guide covers local Banna development and a safe first generation. Banna
modifies an existing ABP solution; start with a clean Git worktree and review
the generated diff. For the complete TUI and CLI workflow, field schema, UI
targets, themes, localization, regeneration, and troubleshooting, continue with
the step-by-step [User guide](user-guide.md).

## Prerequisites

- Rust 1.97.1 with rust-analyzer (selected by `rust-toolchain.toml`);
- Git;
- an ABP solution for generation workflows; and
- the .NET SDK required by the solution's `global.json` and any mobile
  toolchain required by the selected target.

Banna manages the web-generation tools required by the selected target. If `npm` is not on
`PATH`, it downloads the official Node.js 24 LTS archive for Windows, macOS, or
Linux, verifies its SHA-256 checksum against Node.js' release manifest, and
keeps it in Banna's platform data directory. Banna installs
the latest available `Volo.Abp.Cli` into the same private cache rather than
depending on a global ABP CLI. Vue, vue-select, and MiniExcel are installed
without explicit version constraints.

The .NET SDK is deliberately not downloaded automatically: the correct SDK is
solution-specific and may be pinned by `global.json`. When it is unavailable,
Banna stops before changing project files and reports the missing prerequisite.
No package operation uses `sudo` or requests an administrator password.

When a project root is registered, Banna detects Basic or LeptonX from its Web
module, `.Web.csproj`, or Angular startup/workspace files rather than assigning
Basic unconditionally. Existing history metadata is reconciled with the project
files. The detected theme can still be changed later from the TUI project
metadata screen or with `banna-cli project configure --theme`. For Angular,
Banna switches the unversioned theme package, startup providers/modules,
workspace style bundles, and optional footer integration, then requires a
successful production build before saving the new theme metadata. Repeating an
explicit theme selection safely retries an interrupted conversion. Commercial
Angular LeptonX is detected but is not rewritten as LeptonX Lite.

Angular entity generation also installs the same light/dark ThemeChanger
capability used by MVC generation into the shared ABP navigation toolbar. The
selection is stored under the same browser key and updates both Bootstrap's
`data-bs-theme` attribute and LeptonX appearance classes. Bootstrap overrides
(`modern`, `nord`, and `solarized`) are emitted as a Banna-owned Angular style
bundle, leaving the application's `src/styles.scss` untouched. Selecting
`none` removes that bundle, and every Angular override change must pass a
production build before its metadata is saved.

Set `BANNA_TOOLCHAIN_DIR` when automation needs an isolated or pre-provisioned
tool cache:

```sh
BANNA_TOOLCHAIN_DIR=/tmp/banna-tools banna-cli entity generate --help
```

On PowerShell, use `$env:BANNA_TOOLCHAIN_DIR = 'C:\banna-tools'`. Tool download,
installation, spawn, and non-zero-exit failures include the command, working
directory, exit status, and stderr (or stdout when stderr is empty).

Clone the repository and validate the binaries:

```sh
cargo build --locked --bins
cargo run --locked -- --help
cargo run --locked --bin banna-cli -- --help
```

Run the terminal interface with:

```sh
cargo run --locked
```

## Safe First Generation

Use the headless CLI's dry-run mode before writing to a project:

```sh
cargo run --locked --bin banna-cli -- entity generate \
  --project /path/to/project \
  --domain Acme.Billing \
  --namespace Acme.Billing.Invoices \
  --entity Invoice \
  --fields examples/invoice-fields.json \
  --ui razor \
  --dry-run \
  --non-interactive
```

Remove `--dry-run` only after reviewing the preview. Use `--no-merge` when you
want generated files without edits to existing registrations. Use
`--run-migration` only against a development database you intend to change.

Set `BANNA_CONFIG_DIR` to an isolated directory in automation so project-index
state does not leak between jobs:

```sh
BANNA_CONFIG_DIR=/tmp/banna-config cargo test --locked --all-targets
```

## Development

Run the complete local quality gate from the repository root:

```sh
sh scripts/check.sh
```

See [Architecture](architecture.md) before moving responsibilities between
modules and [Contributing](../CONTRIBUTING.md) before opening a pull request.
