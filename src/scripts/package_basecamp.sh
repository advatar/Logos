#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT/app/basecamp-forum"
OUT_DIR="${1:-$ROOT/dist/basecamp}"
PACKAGE_NAME="lp0016-anon-forum-demo.lgx"
PLATFORMS="${BASECAMP_PLATFORMS:-darwin-arm64 darwin-x86_64 linux-aarch64 linux-x86_64}"
LGX_BIN="${LGX_BIN:-}"
if [[ -z "$LGX_BIN" && -x /tmp/logos-package-install/bin/lgx ]]; then
  LGX_BIN="/tmp/logos-package-install/bin/lgx"
fi

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
  cp "$APP_DIR/icon.svg" "$variant_dir/icon.svg"
done

package="$OUT_DIR/$PACKAGE_NAME"
tar -C "$tmp" -czf "$package" manifest.json variants

# When the LGX CLI is available, normalize the package through it so hashes,
# view entries, and variant metadata match Basecamp's package-manager validator.
if [[ -n "$LGX_BIN" && -x "$LGX_BIN" ]]; then
  lgx_lib_dir="$(cd "$(dirname "$LGX_BIN")/../lib" 2>/dev/null && pwd || true)"
  for platform in $PLATFORMS; do
    if [[ -n "$lgx_lib_dir" ]]; then
      DYLD_LIBRARY_PATH="${DYLD_LIBRARY_PATH:-}:$lgx_lib_dir" "$LGX_BIN" add "$package" \
        --variant "$platform" \
        --files "$tmp/variants/$platform" \
        --main Main.qml \
        --view Main.qml \
        --yes >/dev/null
    else
      "$LGX_BIN" add "$package" \
        --variant "$platform" \
        --files "$tmp/variants/$platform" \
        --main Main.qml \
        --view Main.qml \
        --yes >/dev/null
    fi
  done
fi
printf '%s\n' "$package"
