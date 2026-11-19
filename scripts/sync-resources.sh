#!/bin/sh
# Copies Datafaker's locale YAML files (upstream/datafaker/src/main/resources/**/*.yml) into resources/, verbatim.
# build.rs embeds resources/ into the crate, so the crate builds without the upstream submodule.
# tests/resources.rs fails when the copy and upstream differ.
set -eu
cd "$(dirname "$0")/.."
SRC=upstream/datafaker/src/main/resources
rm -rf resources
mkdir -p resources
(cd "$SRC" && find . -name '*.yml' -type f) | while read -r f; do
  mkdir -p "resources/$(dirname "$f")"
  cp "$SRC/$f" "resources/$f"
done
