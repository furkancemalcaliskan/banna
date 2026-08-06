# User Guide

This guide explains how to use Banna against an existing ABP solution. It is
for application developers; repository maintainers should also read
[Getting started](getting-started.md) and [Architecture](architecture.md).

## 1. Before You Start

Banna changes source files and can run package, build, and migration commands.
Use a clean Git worktree so every generated change can be reviewed or reverted.

You need:

- an existing ABP solution whose root contains `src/`;
- the .NET SDK selected by that solution, including `global.json` when present;
- Git; and
- Rust 1.97.1 only when building Banna from source.

Banna manages the latest ABP CLI in its private tool cache. For web generation
it uses the system npm or a checksum-verified private Node.js 24 LTS download.
Generated npm and NuGet dependencies are added without an explicit version
lock. Banna does not install the .NET SDK and never uses `sudo`.

Build both interfaces from the repository root:

```sh
cargo build --release --locked --bins
```

The resulting programs are:

```text
target/release/banna       interactive terminal interface
target/release/banna-cli   non-interactive CLI for scripts and CI
```

The two programs call the same generation and configuration workflows. A
feature available in one does not use a separate set of templates in the other.

## 2. Safest First Run

Start by inspecting the project without modifying it:

```sh
./target/release/banna-cli project inspect \
  --path /path/to/Acme.BookStore \
  --format json
```

Then preview generation with `--dry-run`:

```sh
./target/release/banna-cli entity generate \
  --project /path/to/Acme.BookStore \
  --domain Acme.BookStore \
  --namespace Acme.BookStore.Books \
  --entity Book \
  --fields examples/book-fields.json \
  --ui angular \
  --dry-run \
  --non-interactive
```

Dry-run uses an isolated copy. It does not change the source project, install
packages, update Banna's project index or history, or run migrations. Remove
`--dry-run` after reviewing the report.

## 3. Interactive TUI Workflow

Start the terminal interface:

```sh
./target/release/banna
```

The normal workflow is:

1. Press `n` on the Projects screen.
2. Enter the absolute root path of the ABP solution.
3. Press `Enter` on the registered project.
4. Press `n` on the Entities screen.
5. Enter the entity name. Banna infers the root domain and suggests a plural
   namespace.
6. Add fields one at a time. An empty field name finishes the field list.
7. Select Razor, Vue, Angular, or no UI. Only targets detected in the project
   are offered.
8. Choose whether to run the database migration.
9. Review the generation log and the resulting Git diff.

Press `?` anywhere in the TUI to open the complete shortcut help. The most
frequently used keys are:

| Screen | Key | Result |
| --- | --- | --- |
| Projects | `n` | Register a project |
| Projects | `Enter` | Open the selected project |
| Projects | `E` | Change theme, Bootstrap override, or mobile UI |
| Projects | `Delete` | Remove only the Banna index entry |
| Entities | `n` | Generate a new entity |
| Entities | `Enter` | Regenerate from saved history |
| Entities | `e` | Edit fields and regenerate |
| Entities | `E` | Edit namespace/UI metadata and regenerate |
| Entities | `d` | Remove only the history entry |
| Lists | `j` / `k` | Move down/up |
| Lists | `gg` / `G` | Move to first/last item |
| Editors | `s` | Save and apply |
| Anywhere | `q` / `Esc` | Go back or cancel |

Deleting a project or entity from Banna never deletes generated application
files. Use Git or ordinary file operations when application code must be
removed.

## 4. Field Definition JSON

The headless CLI receives fields as a JSON array. For example:

```json
[
  {
    "name": "Name",
    "type": "string",
    "required": true,
    "max_length": 200,
    "filterable": true,
    "show_in_ui": true
  },
  {
    "name": "Description",
    "type": "textarea",
    "required": false,
    "filterable": true,
    "show_in_ui": true
  },
  {
    "name": "AuthorId",
    "type": "Guid",
    "required": true,
    "navigation": "Acme.BookStore.Authors.Author",
    "navigation_display": "Name",
    "filterable": true,
    "show_in_ui": true
  }
]
```

Supported wizard types are `string`, `int`, `long`, `Guid`, `bool`,
`DateTime`, `textarea`, `decimal`, `DateOnly`, `TimeOnly`, and `enum`.

| Property | Meaning |
| --- | --- |
| `name` | Valid C# identifier; field names are case-insensitively unique |
| `type` | Field type using the spelling shown above |
| `required` | Adds required domain/DTO/UI validation |
| `max_length` | Optional positive maximum for `string` and `textarea` |
| `filterable` | Includes the field in generated filtering contracts and UI |
| `show_in_ui` | Includes the field in generated forms and tables |
| `navigation` | Optional fully-qualified entity or enum type |
| `navigation_display` | Property displayed for a navigation lookup |

Do not declare ABP infrastructure fields such as `Id`, `ConcurrencyStamp`,
`ExtraProperties`, audit properties, soft-delete properties, or `FilterText`.
Banna rejects reserved, duplicate, and unsafe identifiers before writing.

## 5. Generate From the CLI

The complete shape of the command is:

```sh
banna-cli entity generate \
  --project /path/to/project \
  --domain Acme.BookStore \
  --namespace Acme.BookStore.Books \
  --entity Book \
  --fields /path/to/book-fields.json \
  --ui angular \
  --mobile-ui none \
  --non-interactive
```

`--ui` accepts `none`, `razor`, `vue`, or `angular`. `--mobile-ui` accepts
`none` or `react-native`.

Useful safety and automation switches are:

- `--dry-run`: preview without persistent or external side effects;
- `--no-merge`: write generated files without editing existing registration
  files;
- `--run-migration`: add and apply an EF Core migration; use only against a
  development database you intend to change; and
- `--format json`: keep the result machine-readable on stdout while command
  diagnostics remain on stderr.

After a successful real generation, inspect both builds that exist in the
solution, for example:

```sh
dotnet build Acme.BookStore.sln
cd angular
npm run build
```

## 6. What Each UI Target Produces

All UI targets use the same entity/application contracts and permissions.
Target-specific files are generated only when the corresponding host is
detected.

### Razor Pages

Razor generation creates the MVC CRUD page, modal/view models, JavaScript and
styles, resource/menu integration, Excel export, filtering, sorting, paging,
selection, validation, and permissions required by the screen.

### Vue in the MVC Web Host

Vue generation creates the Vue CRUD component and helpers while retaining the
ABP MVC host. Resource mappings and Web module integration are updated
idempotently. When an entity is changed from Razor to Vue through Edit Meta,
Banna removes that entity's obsolete Razor ViewModel mappings from AutoMapper
or Mapperly files so the Web project does not retain stale build references.

### Angular

Angular generation creates a standalone CRUD feature, routes, menu
registration, typed REST proxy files, filtering, sorting, external paging,
navigation lookups, validation, concurrency-safe editing, Excel export, and
bulk deletion.

The generated table footer is always visible, including for an empty result.
Its page size, result range, and navigation labels use ABP localization keys.
Banna adds missing English keys to:

```text
src/<Domain>.Domain.Shared/Localization/<Resource>/en.json
```

Existing localization values are never overwritten. This allows an application
to keep custom wording while newly required keys are added automatically.

Angular generation also installs the shared toolbar ThemeChanger. The choice
is stored in the browser and updates Bootstrap light/dark state and LeptonX
appearance classes.

## 7. Theme Detection and Configuration

When a project is registered or inspected, Banna detects Basic or LeptonX from
the MVC Web module/project and Angular startup/workspace files. Saved metadata
is reconciled with what the project actually contains.

Inspect the result before changing it:

```sh
banna-cli project inspect --path /path/to/Acme.BookStore
```

Switch to LeptonX Lite:

```sh
banna-cli project configure \
  --path /path/to/Acme.BookStore \
  --theme lepton-x \
  --dry-run

banna-cli project configure \
  --path /path/to/Acme.BookStore \
  --theme lepton-x \
  --non-interactive
```

Switch back with `--theme basic`. For Angular, Banna changes the unversioned
theme package, startup providers/modules, workspace styles, and footer wiring.
It saves the new metadata only after a successful production build. Repeating
the same explicit selection safely retries an interrupted conversion.

Commercial Angular LeptonX can be detected, but Banna does not silently replace
or rewrite it as LeptonX Lite.

Apply an optional Bootstrap appearance:

```sh
banna-cli project configure \
  --path /path/to/Acme.BookStore \
  --bootstrap-override nord
```

Available values are `none`, `modern`, `nord`, and `solarized`. Angular
overrides live in a Banna-owned style bundle; the application's
`angular/src/styles.scss` remains owned by the application. Selecting `none`
removes only Banna's override bundle.

## 8. Edit and Regenerate

Banna stores entity definitions in the project's
`.history/codegen_history.json` file. List them with:

```sh
banna-cli history list --project /path/to/Acme.BookStore
```

Regenerate the exact saved definition:

```sh
banna-cli entity regenerate \
  --project /path/to/Acme.BookStore \
  --entity Book \
  --dry-run \
  --non-interactive
```

Remove `--dry-run` after review. In the TUI, `e` edits fields and `E` edits
namespace/UI metadata before regeneration. UI switching also performs
target-specific reconciliation such as removing obsolete Razor ViewModel mapper
references when moving an entity to Vue. Review the diff for screen files that
are no longer part of the selected target.

To forget history without deleting source files:

```sh
banna-cli history remove \
  --project /path/to/Acme.BookStore \
  --entity Book
```

## 9. Project Index and Automation

Register, list, and remove project references:

```sh
banna-cli project add --path /path/to/Acme.BookStore --name BookStore
banna-cli project list --format json
banna-cli project remove BookStore
```

Project removal changes only Banna's global index. Set `BANNA_CONFIG_DIR` to
isolate that index in CI:

```sh
BANNA_CONFIG_DIR=/tmp/banna-config banna-cli project list --format json
```

Set `BANNA_TOOLCHAIN_DIR` to isolate or pre-provision managed Node.js and ABP
CLI tools:

```sh
BANNA_TOOLCHAIN_DIR=/tmp/banna-tools banna-cli entity generate --help
```

## 10. Troubleshooting

### The command is not found

When Banna has only been built from source, it is not automatically added to
`PATH`. Use `./target/release/banna` or `./target/release/banna-cli`, or copy the
release artifact using your normal installation policy.

### The .NET runtime or SDK is missing

Install the SDK selected by the target solution's `global.json`. A newer SDK
does not necessarily provide an older runtime required by a tool. Banna reports
the requested framework and the detected runtimes rather than modifying the
machine globally.

### npm, ABP CLI, or package installation fails

Read the full log. Failures include the command, working directory, exit code,
and stderr. Check network/proxy access, the package registry, filesystem
permissions, and the solution's SDK. Banna package commands do not use explicit
dependency versions.

### Angular theme configuration is not saved

Angular theme and Bootstrap changes require a successful production build.
Fix the reported Angular build error and repeat the same configuration command;
the operation is designed to be retry-safe.

### Existing custom localization disappeared

Banna adds a localization key only when that exact key is absent from `Texts`.
It does not replace an existing value. Check the application's culture file and
the resource prefix used by the generated template.

### Generated code does not compile

Run generation from a clean branch, keep the complete Banna log, and review the
diff before manual edits. Confirm that the solution layout matches a supported
ABP host and run both the .NET and selected frontend production builds. When
reporting a problem, include Banna's version, operating system, ABP/.NET/Node
versions, selected UI/theme, command, log, and the smallest reproducible entity
definition. See [Support](../SUPPORT.md) and [Security](../SECURITY.md).

## 11. Where to Go Next

- Run `banna-cli <command> --help` for authoritative option names.
- Read [Getting started](getting-started.md) for toolchain and development setup.
- Read [Architecture](architecture.md) before changing generator internals.
- Read [Generated-code licensing](generated-code-licensing.md) for generated
  output rights.
- Review [Support](../SUPPORT.md) before filing a reproducible bug report.
