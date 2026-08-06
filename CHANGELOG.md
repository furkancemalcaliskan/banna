# Changelog

All notable changes to Banna will be documented in this file. The format is
based on Keep a Changelog, and releases follow Semantic Versioning.

## Unreleased

### Added

- `banna-cli` for non-interactive project, history, configuration, generation,
  regeneration, migration, and dry-run workflows.
- Apache-2.0 licensing, contributor and security guidance, architecture
  documentation, CI, dependency update automation, and release checks.
- Open-source governance for human and agent contributors, including the
  `develop` integration and `main` release branch contract, repository policy
  enforcement, issue and pull-request templates, community/support policies,
  a maintainer guide, and a reproducible local quality gate.
- ABP Framework rel-8.3 attribution, LGPL-3.0-only license propagation, and
  third-party notices for the modified React Native scaffold.
- An unrestricted generated-output permission for Banna's original templates,
  including proprietary use without attribution.
- Characterization coverage for supported UI generators and idempotent merge
  behavior.
- Automated Cargo license, source, duplicate-version, and RustSec advisory
  policy checks.
- Cross-platform managed tooling that verifies and caches portable Node.js 24
  LTS when npm is missing, and installs the latest available ABP CLI into a
  private cache.
- A step-by-step user guide covering the TUI, CLI, field schema, generated UI
  targets, themes, localization, regeneration, automation, and troubleshooting.

### Changed

- Renamed the product, Cargo package, executable, configuration identity, and
  documentation from the previous internal name to Banna.
- Split application use cases, persistence/process ports and adapters, TUI
  state/render/input concerns, target generators, and merge concerns into
  focused modules.
- Replaced hidden workflow state and production panic assumptions with typed,
  explicit state and contextual errors.
- Updated the TUI stack to Ratatui 0.30 and Crossterm 0.29, raised the minimum
  supported Rust version to 1.97.1, and removed the unmaintained `paste`
  dependency.
- Updated `anyhow` to a version containing the RUSTSEC-2026-0190 soundness fix.
- Pinned development and CI to Rust 1.97.1 with its matching rust-analyzer and
  aligned release validation with stable SemVer tags and dated changelog entries.
- Removed explicit version constraints from generated-project dependencies and
  the managed ABP CLI,
  removed privilege/password prompts, and unified TUI and CLI command execution.

### Fixed

- Generate a self-contained standalone Angular CRUD feature, including typed
  REST proxies, permissions, paging/sorting/filtering, navigation lookups,
  validation, concurrency-safe editing, Excel export, and bulk deletion.
- Support current and legacy Angular route/menu registration layouts without
  corrupting `tsconfig.json` or depending on a generated shared module.
- Detect Basic and LeptonX themes directly from Angular startup code and
  workspace manifests when a project has no MVC Web host.
- Reconcile stale project-history theme metadata when the TUI starts, registers,
  opens, or edits a project so the project list reflects the detected theme.
- Switch Angular projects bidirectionally between Basic and LeptonX Lite by
  updating unversioned npm dependencies, standalone or legacy startup wiring,
  workspace style bundles, and optional footer integration, with a required
  production build and retry-safe explicit selections.
- Generate a shared-toolbar Angular ThemeChanger for Basic and LeptonX, and
  apply removable Modern, Nord, or Solarized Bootstrap overrides through a
  Banna-owned style bundle without replacing application styles.
- Render an always-visible MVC-parity Angular pagination footer with page size,
  result range, and disabled navigation controls even for empty result sets.
- Clean up entity-scoped Razor ViewModel mappings from both AutoMapper profiles
  and Mapperly fallback files when an MVC entity is switched to Vue.
- Removed duplicate Vue filter state, escaped generated DataTable text columns,
  debounced advanced-filter reloads, and made navigation lookups honor their
  configured display fields.
- Hardened generated Vue modal lifecycle handling against duplicate mounts,
  failed asynchronous initialization, and duplicate save submissions.
- Made Vue helper and ABP bundle integration updates idempotent and fail safely
  when an unknown WebModule layout cannot be patched.
- Rejected unsafe, reserved, and duplicate generation identifiers before
  writing project files.
- Restored generation of the embedded Razor global stylesheet by using its
  actual asset path.
- Added contextual command failure diagnostics and native Linux, macOS, and
  Windows test coverage for platform-specific command resolution.
- Detect the configured ABP Basic or LeptonX theme from the Web module/project
  instead of preconfiguring every registered project as Basic.

## 0.1.0

Not yet released.
