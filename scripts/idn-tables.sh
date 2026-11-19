#!/bin/sh
# Regenerates src/idn_tables.rs and tests/vectors/idn.tsv from the JDK's java.net.IDN (scripts/IdnTables.java).
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR=$(mktemp -d)
javac -d "$DIR" "$ROOT/scripts/IdnTables.java"
java -cp "$DIR" IdnTables "$ROOT/src/idn_tables.rs" "$ROOT/tests/vectors/idn.tsv"
rm -rf "$DIR"
