#!/bin/sh
# Regenerates src/java/time/tzdb.bin (scripts/TzTables.java): a JDK's tzdb rules and en_US zone names.
#   scripts/tz-tables.sh                               the JDK on the PATH
#   scripts/tz-tables.sh mockserver/mockserver:8.0.0   that image's JVM (Docker; the class is compiled locally)
# Recorded from MockServer 8.0.0's image: Temurin 26.0.2, tzdb 2026b (README, "Differences").
set -eu
cd "$(dirname "$0")/.."
IMAGE="${1:-}"
OPEN="--add-exports java.base/sun.util.locale.provider=ALL-UNNAMED"
RUN="--add-opens java.base/java.time.zone=ALL-UNNAMED --add-opens java.base/java.time.format=ALL-UNNAMED"
PROPS="-Duser.language=en -Duser.country=US"
if [ -z "$IMAGE" ]; then
  DIR=$(mktemp -d)
  javac $OPEN -d "$DIR" scripts/TzTables.java
  java $OPEN $RUN $PROPS -cp "$DIR" TzTables src/java/time/tzdb.bin
else
  # A Docker VM mounts the home directory, not the system temporary directory. javac refuses --add-exports with
  # --release, so the class is compiled for the local JDK's release, which must not be newer than the image's JVM.
  DIR="$(pwd)/target/tz-tables-image"
  rm -rf "$DIR"
  mkdir -p "$DIR/classes"
  javac $OPEN -d "$DIR/classes" scripts/TzTables.java
  JAVA=$(docker image inspect "$IMAGE" --format '{{index .Config.Entrypoint 0}}')
  docker run --rm -v "$DIR:/record" --entrypoint "$JAVA" "$IMAGE" $OPEN $RUN $PROPS \
    -cp /record/classes TzTables /record/tzdb.bin
  mv "$DIR/tzdb.bin" src/java/time/tzdb.bin
fi
rm -rf "$DIR"
