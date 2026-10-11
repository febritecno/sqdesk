#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
VERSION="$(sed -nE "s/^version[[:space:]]*=[[:space:]]*'([^']+)'.*/\1/p" "$ROOT/build.gradle" | head -n 1)"
PACKAGE_DIR="$ROOT/dist/sqdesk-jdbc-plugin-$VERSION"
ZIP_PATH="$ROOT/dist/sqdesk-jdbc-plugin-$VERSION.zip"
LATEST_ZIP_PATH="$ROOT/dist/sqdesk-jdbc-plugin-latest.zip"

cd "$ROOT"
./gradlew -q shadowJar -x test

rm -rf "$PACKAGE_DIR" "$ZIP_PATH" "$LATEST_ZIP_PATH"
mkdir -p "$PACKAGE_DIR/bin" "$PACKAGE_DIR/lib"
cp "$ROOT/manifest.json" "$PACKAGE_DIR/manifest.json"
cp "$ROOT/bin/sqdesk-jdbc-plugin" "$PACKAGE_DIR/bin/sqdesk-jdbc-plugin"
cp "$ROOT/bin/sqdesk-jdbc-plugin.bat" "$PACKAGE_DIR/bin/sqdesk-jdbc-plugin.bat"
cp "$ROOT/bin/sqdesk-maven-resolver" "$PACKAGE_DIR/bin/sqdesk-maven-resolver"
cp "$ROOT/bin/sqdesk-maven-resolver.bat" "$PACKAGE_DIR/bin/sqdesk-maven-resolver.bat"
cp "$ROOT/build/libs/sqdesk-jdbc-plugin-all.jar" "$PACKAGE_DIR/lib/sqdesk-jdbc-plugin.jar"
chmod +x "$PACKAGE_DIR/bin/sqdesk-jdbc-plugin"
chmod +x "$PACKAGE_DIR/bin/sqdesk-maven-resolver"

(cd "$ROOT/dist" && zip -qr "sqdesk-jdbc-plugin-$VERSION.zip" "sqdesk-jdbc-plugin-$VERSION")
cp "$ZIP_PATH" "$LATEST_ZIP_PATH"
echo "$ZIP_PATH"
echo "$LATEST_ZIP_PATH"
