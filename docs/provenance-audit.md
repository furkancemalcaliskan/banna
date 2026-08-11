# Embedded-content provenance audit

This document is a release gate for content compiled into Banna. It records
what is distributed inside the executable; it does not infer ownership or
license compatibility from Git history alone.

## Scope and result

`rust-embed` packages both `templates/` and `projects/` into the binary. The
current inventory contains 189 files:

- 79 text templates under `templates/`;
- 105 text/configuration, notice, or license files under `projects/`;
- 5 PNG files under `projects/Vanilla/react-native/assets/`.

The React Native upstream, version line, LGPL-3.0-only license, modification
notice, and affected directory scope are now recorded and ship inside generated
mobile projects. The maintainer confirms that all 79 files under `templates/`
are original expressions of Banna's opinionated generation style and were not
copied from ABP Suite or another generator. The five PNGs are derivatives of
the author-provided official Banna logo; their source and reproduction process
are recorded in the [brand asset guide](brand-assets.md).

The embedded-content provenance gate is satisfied. Repeat the inventory after
the final packaged-file list is produced.

## Confirmed open-source React Native origin

The maintainer confirms that the React Native scaffold was taken from and
customized from this public upstream source:

- <https://github.com/abpframework/abp/tree/rel-8.3/templates/app/react-native>
- Version line: `rel-8.3`
- Repository license expression: `LGPL-3.0-only`, explicitly declared by
  [`rel-8.3/common.props`](https://github.com/abpframework/abp/blob/rel-8.3/common.props)
- License text: upstream
  [`rel-8.3/LICENSE.md`](https://github.com/abpframework/abp/blob/rel-8.3/LICENSE.md)
- Local scope: `projects/Vanilla/react-native/`, excluding Banna artwork and
  local notice/license files

The canonical GNU LGPLv3 text is stored at `LICENSES/LGPL-3.0-only.txt`; the
incorporated GPLv3 text is stored at `LICENSES/GPL-3.0-only.txt`. Both are copied
into the scaffold. Attribution and a summary of modifications are stored in
`THIRD_PARTY_NOTICES.md` and the scaffold-local `NOTICE.md`. The Cargo package
uses the conservative SPDX expression `Apache-2.0 AND LGPL-3.0-only` because it
distributes both scopes.

The upstream repository is published by the `abpframework` GitHub organization
and describes the project as community-driven. Copyright remains with the
respective upstream authors, contributors, and any other named holders, as
recorded by upstream source notices and repository history; no single corporate
copyright holder is inferred here.

This resolves the earlier concern that the mobile scaffold might be sourced
from ABP Commercial.

Recorded on 2026-08-06 from the maintainer-provided upstream link, license text,
and authorship statement for Banna's generator templates.

## Binary asset inventory

| File | Dimensions | SHA-256 | Status |
| --- | ---: | --- | --- |
| `adaptive-icon.png` | 1024 x 1024 | `a647ce6cddb5bfb9e1f42d7f86713f694b2195926bff0812eda2e7305332bf4c` | Official Banna derivative; recorded |
| `avatar.png` | 1024 x 1024 | `7646815876feb878b9a0956237f939e20e1bacb9446299b747edc3503571fa0f` | Official Banna derivative; recorded |
| `icon.png` | 192 x 192 | `e70b73e8a4230c040527caee577b106a51647159fb72755d4bb9b7fcb7527064` | Official Banna derivative; recorded |
| `logo.png` | 256 x 256 | `623411f895bb0bb0a3e51fe057990bb520cb7e5ac89828fde8d7bb6d4affa863` | Official Banna derivative; recorded |
| `splash.png` | 1242 x 2436 | `bd71ee69a21b070f7ae113101a344ee8d7b68fef4d27aa7f70bda155bb41a1a2` | Official Banna derivative; recorded |

The five files are deterministic derivatives of `assets/brand/banna.png`. The
icon, adaptive icon, and splash are referenced by Expo configuration; the logo
and avatar are referenced by application components. The Expo background colors
were updated to complement the official artwork at the same time.

## Dependency boundary

Rust crates and packages declared by the embedded React Native `package.json`
are dependencies, not copied source in this inventory. Their licenses still
need normal dependency-policy review, but they are distinct from the embedded
files above. Generated applications resolve JavaScript packages through their
package manager; Banna does not vendor those packages into this repository.

## Recording a resolution

For an original file, record the author and confirmation date in this document.
For a third-party file, record its canonical source URL, exact version or
revision, copyright holder, license identifier, and any notice text that must
ship. Do not mark this gate complete solely because a file lacks a license
header.
