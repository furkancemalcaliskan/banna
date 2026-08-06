#!/bin/sh
set -eu

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

node --test "$repository_root/scripts/repository-policy.test.mjs"
node "$repository_root/scripts/repository-policy.mjs" --root "$repository_root" "$@"

