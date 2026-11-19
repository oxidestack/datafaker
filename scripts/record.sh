#!/bin/sh
# Re-records a vector file from Datafaker 2.7.0: scripts/record.sh tests/vectors/<file>.tsv [Recorder]
# Keeps each row's seed and call columns and asks Datafaker again, through scripts/<Recorder>.java (default
# Vectors; Transformations records tests/vectors/transformations.tsv). Needs a JDK and the jars in ~/.m2
# (mvn dependency:get -Dartifact=net.datafaker:datafaker:2.7.0 fetches them). The JVM runs in en_US and in GMT, the
# FORMAT locale and default zone this crate assumes (README, "Differences"), or in the zone a `# zone <id>` line of
# the file names (tests/vectors.rs replays that file with java::time::with_default_zone).
# A `# image <docker image>` line runs the recorder on that image's JVM instead of the local one (Docker; the class
# is compiled locally with --release 25): `# image mockserver/mockserver:8.0.0` records on the JVM MockServer 8.0.0
# runs (Temurin 26.0.2), for behaviour that differs between JDK releases (README, "Differences").
set -eu
M="$HOME/.m2/repository"
JARS="$M/net/datafaker/datafaker/2.7.0/datafaker-2.7.0.jar $M/org/yaml/snakeyaml/2.6/snakeyaml-2.6.jar $M/com/github/curious-odd-man/rgxgen/3.1/rgxgen-3.1.jar $M/com/googlecode/libphonenumber/libphonenumber/9.0.41/libphonenumber-9.0.41.jar"
CP=$(echo $JARS | tr ' ' ':')
CLASS="${2:-Vectors}"
ZONE=$(sed -n 's/^# zone //p' "$1" | head -n 1)
IMAGE=$(sed -n 's/^# image //p' "$1" | head -n 1)
PROPS="-Djava.awt.headless=true -Duser.timezone=${ZONE:-GMT} -Duser.language=en -Duser.country=US"
if [ -z "$IMAGE" ]; then
  DIR=$(mktemp -d)
  javac -nowarn -d "$DIR" -cp "$CP" "$(dirname "$0")/$CLASS.java"
  cut -f1,2 "$1" | java $PROPS -cp "$CP:$DIR" "$CLASS" > "$1.new"
else
  # A Docker VM mounts the home directory, not the system temporary directory.
  DIR="$(cd "$(dirname "$0")/.." && pwd)/target/record-image"
  rm -rf "$DIR"
  mkdir -p "$DIR/jars" "$DIR/classes"
  cp $JARS "$DIR/jars/"
  javac -nowarn --release 25 -d "$DIR/classes" -cp "$CP" "$(dirname "$0")/$CLASS.java"
  JAVA=$(docker image inspect "$IMAGE" --format '{{index .Config.Entrypoint 0}}')
  cut -f1,2 "$1" | docker run --rm -i -v "$DIR:/record" --entrypoint "$JAVA" "$IMAGE" $PROPS \
    -cp "/record/jars/*:/record/classes" "$CLASS" > "$1.new"
fi
mv "$1.new" "$1"
rm -rf "$DIR"
