# Dependency policy

Banna checks the complete Cargo dependency graph with `cargo-deny`. The
repository configuration is the source of truth:

- dependencies must come from crates.io; unreviewed registries and Git sources
  are rejected;
- wildcard dependency requirements are rejected;
- known RustSec advisories are rejected unless a documented advisory ID is
  explicitly added to `deny.toml`;
- permissive licenses in the `deny.toml` allowlist are accepted;
- CDLA-Permissive-2.0 is accepted for the public root-certificate data shipped
  by the HTTPS client dependency;
- LGPL-3.0-only is an exception limited to the local `banna` package because
  the distributed crate embeds the attributed ABP React Native scaffold; and
- duplicate transitive versions are reported as warnings so they remain
  visible without making upstream compatibility splits a release blocker.

Run the same release gate locally with:

```sh
cargo deny check
```

An exception must identify its exact crate or advisory and explain why it is
necessary in the pull request that introduces it. Broad registry, Git, license,
or advisory exceptions are not accepted.
