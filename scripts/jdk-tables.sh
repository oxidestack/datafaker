#!/bin/sh
# Regenerates src/providers/jdk_tables.rs from the JDK on the PATH and datafaker-2.7.0.jar (scripts/JdkTables.java).
set -eu
cd "$(dirname "$0")/.."
CP="$HOME/.m2/repository/net/datafaker/datafaker/2.7.0/datafaker-2.7.0.jar"
DIR=$(mktemp -d)
javac -d "$DIR" -cp "$CP" scripts/JdkTables.java
java -cp "$CP:$DIR" JdkTables > src/providers/jdk_tables.rs
rm -rf "$DIR"
