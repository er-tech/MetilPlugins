#!/bin/sh
# Packs one plugin as out/<plugin>-<version>-<platform>.mtp for the release assets.
# Usage: tools/pack-plugin.sh <plugin folder name> [rust|python]; SMITH_PLATFORM and SMITH_VERSION are
# set by `smith release assets`. With SMITH_VERSION the package says that version in its plugin.json (the
# folder in the repository keeps the version it has); without it the folder is packed as it is.
# metil is private: $METIL is the Metil checkout the pipeline puts beside this repository.
set -e
app="$1"; kind="${2:-rust}"
cargo build --release --manifest-path "$METIL/Cargo.toml" -p metil-cli
metil="$METIL/dist/cargo/release/metil"
[ -f "$metil.exe" ] && metil="$metil.exe"
mkdir -p out

folder="apps/$app"
if [ -n "$SMITH_VERSION" ]; then
  stage="$(mktemp -d)"
  trap 'rm -rf "$stage"' EXIT
  cp -R "apps/$app" "$stage/$app"
  folder="$stage/$app"
  # The version is on one of the first lines of the manifest, after the name.
  sed -E "1,6s/(\"version\"[[:space:]]*:[[:space:]]*\")[^\"]*(\")/\1$SMITH_VERSION\2/" "$folder/plugin.json" > "$folder/plugin.json.stamped"
  mv "$folder/plugin.json.stamped" "$folder/plugin.json"
  grep -q "\"version\": \"$SMITH_VERSION\"" "$folder/plugin.json" || {
    echo "the manifest of $app could not be stamped with version $SMITH_VERSION" >&2
    exit 1
  }
fi

if [ "$kind" = python ]; then
  "$metil" plugins pack "$folder" --platform any --out out
else
  crate="metilplugins-$app"
  cargo build --release -p "$crate"
  program="dist/cargo/release/$crate"
  [ -f "$program.exe" ] && program="$program.exe"
  "$metil" plugins pack "$folder" --program "$program" --out out
fi
