#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
  echo "usage: scripts/release-policy.sh vMAJOR.MINOR.PATCH" >&2
  exit 2
fi

release_ref=$1
if ! printf '%s\n' "$release_ref" | grep -Eq '^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'; then
  echo "release policy: tag must be stable SemVer in the form vMAJOR.MINOR.PATCH" >&2
  exit 1
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
version=${release_ref#v}
manifest_version=$(sed -n '/^\[package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' "$repository_root/Cargo.toml" | head -n 1)

if [ "$manifest_version" != "$version" ]; then
  echo "release policy: Cargo.toml version $manifest_version does not match $release_ref" >&2
  exit 1
fi

escaped_version=$(printf '%s' "$version" | sed 's/\./\\./g')
if ! grep -Eq "^## \[$escaped_version\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" "$repository_root/CHANGELOG.md"; then
  echo "release policy: CHANGELOG.md needs a dated [$version] release heading" >&2
  exit 1
fi

release_notes="$repository_root/docs/releases/$release_ref.md"
if [ ! -s "$release_notes" ]; then
  echo "release policy: release notes are missing or empty: docs/releases/$release_ref.md" >&2
  exit 1
fi

first_content_line=$(sed -n '/[^[:space:]]/{p;q;}' "$release_notes")
if [ "$first_content_line" != "# Banna $release_ref" ]; then
  echo "release policy: release notes must begin with: # Banna $release_ref" >&2
  exit 1
fi

echo "release policy: ok ($release_ref)"
