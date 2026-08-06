# Technical-debt register

This register records observed structural risks without presenting unfinished
work as implemented.

## High priority

### Split generation by target

The original generator combined shared mapping, C# generation, every UI target,
theme updates, JSON editing, and external commands. Angular, Razor, Vue, and
React Native entity generation now live in focused target modules.
Scaffold/configuration and project metadata mutation live in dedicated
`vue_workspace` and `project_meta` modules, with port-based command execution in
shared support. A target-neutral immutable generation context avoids repeating
project/entity parameter chains without absorbing runtime policy. The remaining
risk is the large common C# fragment compiler; preserve output fixtures before
decomposing it further.

### Separate TUI state transitions from rendering

`src/tui.rs` combines rendering, modal state, input parsing, persistence,
background jobs, and workflow rules. Extract pure transition functions and
workflow-specific modules before changing the visual design. This is the main
barrier to useful unit tests.

Initial extraction is in place for choices, console state, background-job
messages, layout, navigation, modal/state types, rendering, and input dispatch.
Field/meta editing and screen-level key dispatch have focused handlers. Entity
creation now lives in a terminal-independent state machine with transition
tests. Workflow completion uses typed `App`-owned state, and console callbacks
return typed prompts. The previous thread-local mailboxes and next-hint handoff,
duplicated completion consumers, and unproduced legacy edit paths are removed.
Remaining work is to reduce state-dependent assumptions in root orchestration.
Production TUI paths no longer use `unwrap`/`expect` for selected project,
history, confirmation, or render state, and direct project/entity selection is
bounds checked.

### Extend generated-output fixtures

Representative generated-output tests now cover Razor, Vue, Angular, and React
Native targets through the real generation context. Merge characterization
tests cover core C#, Razor/Web, Mapperly fallback, and repeated byte-stable
application. Continue expanding field-type and project-layout combinations
before changing the common C# fragment compiler or text-mutation strategy.

## Medium priority

### Replace state-dependent panics

Several `unwrap` and `expect` calls assume valid TUI state or a particular JSON
shape. Some regex construction unwraps are safe because patterns are static,
but user/project-derived assumptions should return contextual errors. Address
these incrementally alongside tests for the affected workflow.

### Centralize structured-file edits

JSON and source-file mutation is spread across generator functions. Introduce
small read/validate/write helpers with atomic replacement, while retaining the
format expected by generated projects.

The merge path now validates required mappings without `HashMap` indexing
panics, and its core C# mutations have idempotency characterization coverage.
Application, Contracts/Permissions, EF Core, and Web mutations are separated by
destination concern, and regex compilation propagates errors. Remaining merge
coverage now includes Razor/Web and Mapperly fallback idempotency. Broader
generated-output fixtures are still required before changing the text-based
mutation strategy itself.

## Completed baseline cleanup

- Product, executable, package, TUI, and config identities use Banna.
- The binary entry point is separated from the library.
- TUI and headless CLI entity generation share an application use-case boundary.
- `banna-cli entity generate` supports JSON field definitions, human/JSON output,
  no-merge operation, and non-interactive failure behavior.
- Generation and merge output flows through typed report events instead of
  writing progress directly to stdout.
- Project-index persistence implements a repository port and atomic JSON adapter;
  `banna-cli project list/add/remove` use the same index as the TUI.
- Process execution uses an explicit command port and a cross-platform system
  adapter; generator code no longer owns process spawning or privilege prompts.
- EF migrations are an application use case shared by TUI generation and the
  headless `--run-migration` option.
- Project history is accessed through a persistence port and atomic JSON
  adapter; invalid history is reported instead of silently replaced.
- `banna-cli project inspect`, `history list/remove`, and `entity regenerate`
  reuse application workflows and preserve generated files during metadata
  removal.
- TUI and CLI project configuration share one application workflow for theme,
  Bootstrap override, mobile scaffolding, and metadata persistence.
- Entity generation/regeneration and project configuration support isolated
  dry-run execution without source, history, command, or tool-download side effects.
- Persistent project-index concerns are separated from serializable models.
- TUI workflow completions and prompt hints use explicit typed state; no hidden
  thread-local workflow state remains.
- Merge mapping validation is typed; Application, Permissions, EF Core, and Web
  mutations are separated and covered by core/Razor idempotency fixtures.
- Package metadata, scoped Apache-2.0/LGPL licensing, contributor guidance,
  security guidance, architecture documentation, and continuous integration are
  present.
- Generated-output fixtures cover all supported UI targets, alongside merge
  idempotency characterization tests.
- Embedded-content provenance is recorded: all Banna generator templates are
  maintainer-authored Apache-2.0 content with an unrestricted generated-output
  grant; the customized ABP Framework rel-8.3 React Native scaffold retains
  LGPL-3.0-only with upstream attribution, modification notice, and complete
  LGPLv3/GPLv3 license copies.
- Cargo dependency licenses, sources, RustSec advisories, and duplicate
  versions are continuously checked; release builds also verify Rust 1.97.1 as
  the declared MSRV.
- Tagged release automation packages both binaries with notices, license texts,
  and SHA-256 checksums for Linux, macOS, and Windows.
