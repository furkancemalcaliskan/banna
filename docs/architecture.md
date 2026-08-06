# Architecture

Banna is a modular monolith: one publishable Cargo package, a reusable library
boundary, and two delivery binaries. This is intentional. The project is a
focused code generator, so a multi-crate framework layout would add release and
dependency overhead without creating a useful ownership boundary.

## System Context

```text
developer or CI
       |
       +-- banna (TUI) ---------+
       |                        |
       +-- banna-cli -----------+--> application use cases
                                         |
                            +------------+-------------+
                            |                          |
                     generation + merge        ports -> adapters
                            |                          |
                 selected ABP source tree     process / toolchain /
                                              index / history
```

Banna is a development-time tool. It reads and changes a selected ABP source
tree; it is not linked into the generated application's runtime. The selected
project, its dependencies, and any database migration command remain outside
Banna's trust boundary.

## Package And Entry Points

`src/lib.rs` is the composition boundary. The default `banna` binary starts the
terminal interface and `src/bin/banna-cli.rs` starts the headless interface.
Both remain thin and delegate to library entry points.

The library is currently an application boundary, not a promised stable Rust
SDK. Public Rust API compatibility is secondary to the documented binary and
generated-output contracts before `1.0.0`.

## Module Ownership

| Module | Owns | Must not own |
| --- | --- | --- |
| `application` | Requests, validation, use-case sequencing, outcome events | Terminal rendering or direct process spawning |
| `ports` | Contracts for commands, history, and project state | System implementations |
| `adapters` | System/dry-run command runners, managed tools, staging, atomic persistence | Product workflow decisions |
| `cli` | Shared command-line value types and options | Generation behavior |
| `headless` | CLI parsing, human/JSON presentation, exit-facing errors | A second implementation of use cases |
| `tui` | Terminal state, input, rendering, and workflow adaptation | Generator or persistence internals |
| `generator` | Shared generation orchestration and target-neutral context | Terminal or headless presentation |
| `generator/*` | Angular, Razor, Vue, React Native, project metadata, and workspace-specific generation | Unrelated target behavior |
| `merge` | Validation and ordered merge orchestration | Target-specific rendering |
| `merge/*` | Idempotent edits grouped by Application, Permissions, EF Core, and Web concern | Cross-layer orchestration |
| `models` | Serializable project, entity, field, and history data | I/O policy |
| `state` | Project-index locations and atomic JSON adapter | UI state |
| `templates` | Read-only access to compile-time embedded inputs | Runtime downloads |
| `theme` | Banna terminal presentation styles | Generated application themes |
| `helpers`, `utils` | Small naming, rendering, filesystem, and logging helpers | New feature orchestration |

`templates/` contains generated ABP fragments. `projects/` contains the
embedded React Native base. These are product inputs compiled into the binary
with `rust-embed`, not samples. Their contents affect package size, generated
output, security, and licensing.

## Dependency Direction

The intended direction is:

```text
binaries -> delivery adapters -> application -> generator / merge / models
                                      |
                                      +-> ports <- adapters

templates <- generator
theme     <- TUI only
```

Application code may depend on port traits but not concrete system adapters.
Composition selects adapters at the delivery boundary. Generator and merge code
do not render terminal UI or parse Clap commands. Models remain independent of
Ratatui, Dialoguer, and process execution.

Do not split an internal crate merely to imitate a layered repository. Consider
a new crate only when at least one of these is true:

- it is independently published or versioned;
- it needs a materially different dependency or compilation profile;
- it is reused outside Banna; or
- a dependency boundary cannot be expressed and tested within the package.

Record the decision before changing the package topology.

## Generation Boundary

`application::generate_entity` validates an interface-independent request,
selects normal or dry-run adapters, invokes shared generator orchestration, and
returns typed events. Target modules receive a small immutable generation
context with project/output roots and entity identity. Runtime choices such as
UI target, mobile target, merge behavior, migrations, and dry-run stay in outer
request/orchestration layers.

New target-specific generation belongs in the matching focused module. Shared
logic should move into the generator core only after a real second target uses
it. Tests should compare representative output and cover important field types
and host layouts.

## Merge Boundary

Merge operations change existing application files and therefore carry more
risk than creating a new file. The root merge module validates a typed
`MergeContext` before mutation and coordinates destination-specific modules.

A merge operation should:

- fail with context when required project structure is absent;
- avoid partially guessing an unsupported layout;
- preserve unrelated user code and formatting as far as the current text-based
  strategy permits; and
- be byte-stable when applied repeatedly with the same input.

Repeated-application characterization tests are required when adding a merge
path or changing placement rules.

## Side Effects And Persistent Data

The project index is JSON stored under the platform configuration directory
selected by `directories`, using the `banna` application identity.
`BANNA_CONFIG_DIR` overrides that location for isolated automation.

Generation history is stored at `.history/codegen_history.json` inside the
selected project. Index and history writes use temporary files in the same
directory followed by atomic replacement. Removing a project or history entry
changes metadata only; it never deletes generated source files.

External commands are typed command specifications executed through the
command-runner port. Its system adapter resolves platform executables, manages
checksum-verified portable Node.js and an exact private ABP CLI, and captures
stdout, stderr, and exit status. Tool preflight runs before generation mutates
project files. EF migrations are an explicit application workflow, not an
automatic side effect of ordinary generation.

## Dry-Run Contract

Dry-run copies relevant project sources to a temporary workspace and executes
the same generation and merge code there. A simulation adapter replaces
external commands, including managed-tool downloads. Reported paths are
translated back to the selected source root.

A successful dry-run must not modify:

- the selected project;
- generation history;
- the Banna project index;
- a database, package installation, or another external system.

Tests should assert absence of these side effects, not only successful output.

## Output Contracts

Generation reports contain typed `created`, `modified`, `skipped`, `warning`,
and `info` events. The TUI presents them as job output. In JSON mode,
`banna-cli` writes exactly one machine-readable result to stdout and sends
progress and diagnostics to stderr.

Generated output is user-owned source but may be reviewed as a compatibility
surface. User-visible changes belong in the changelog and should be called out
explicitly in pull requests and release notes.

## Licensing And Provenance Boundary

Banna's Rust code and original generator templates are Apache-2.0-licensed,
with an additional unrestricted permission for output derived from original
templates. The embedded React Native base is a modified ABP Framework rel-8.3
derivative under LGPL-3.0-only. See
[Generated-code licensing](generated-code-licensing.md) and
[Third-party notices](../THIRD_PARTY_NOTICES.md).

Every new embedded file must have known provenance. Record its canonical source,
exact version or revision, license, local scope, modifications, and required
notices before release. The [provenance audit](provenance-audit.md) and
[dependency policy](dependency-policy.md) are release gates.

## Architectural Change Process

An architectural change should arrive as one coherent pull request containing:

1. the motivating issue and concrete constraint;
2. implementation and migration of current callers;
3. boundary or characterization tests;
4. updated architecture and contributor guidance; and
5. generated-output, compatibility, security, and licensing impact.

Planned boundaries belong in issues, not in this document. This document
describes the repository that exists.
