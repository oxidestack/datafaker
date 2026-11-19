#!/bin/sh
# Regenerates src/providers/iso_codes.rs from the JDK on the PATH (scripts/IsoCodes.java).
set -eu
cd "$(dirname "$0")/.."
DIR=$(mktemp -d)
javac -d "$DIR" scripts/IsoCodes.java
java -cp "$DIR" IsoCodes > src/providers/iso_codes.rs
rm -rf "$DIR"
