#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT/app/basecamp-forum"
OUT_DIR="${1:-$ROOT/dist/basecamp}"
PACKAGE_NAME="lp0016-anon-forum-demo.lgx"
PLATFORMS="${BASECAMP_PLATFORMS:-darwin-arm64 darwin-x86_64 linux-aarch64 linux-x86_64}"

mkdir -p "$OUT_DIR"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/variants"
cp "$APP_DIR/manifest.json" "$tmp/manifest.json"

for platform in $PLATFORMS; do
  variant_dir="$tmp/variants/$platform"
  mkdir -p "$variant_dir"
  cp "$APP_DIR/Main.qml" "$variant_dir/Main.qml"
  cp "$APP_DIR/metadata.json" "$variant_dir/metadata.json"
done

tar -C "$tmp" -czf "$OUT_DIR/$PACKAGE_NAME" .
printf '%s\n' "$OUT_DIR/$PACKAGE_NAME"
