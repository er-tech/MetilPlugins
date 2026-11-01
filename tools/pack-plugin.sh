#!/bin/sh
# Packs one plugin as out/<plugin>-<version>-<platform>.mtp for the release assets.
# Usage: tools/pack-plugin.sh <plugin folder name> [rust|python]; SMITH_PLATFORM is set by `smith release assets`.
# metil-plugin is private: $METIL is the Metil checkout the pipeline puts beside this repository.
set -e
app="$1"; kind="${2:-rust}"
cargo build --release --manifest-path "$METIL/Cargo.toml" -p metil-plugin
metil-plugin="$METIL/dist/cargo/release/metil-plugin"
[ -f "$metil-plugin.exe" ] && metil-plugin="$metil-plugin.exe"
mkdir -p out
if [ "$kind" = python ]; then
  "$metil-plugin" pack "apps/$app" --platform any --out out
else
  crate="metilplugins-$app"
  cargo build --release -p "$crate"
  program="dist/cargo/release/$crate"
  [ -f "$program.exe" ] && program="$program.exe"
  "$metil-plugin" pack "apps/$app" --program "$program" --out out
fi
