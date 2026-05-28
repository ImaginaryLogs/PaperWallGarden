#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

for crate_dir in "$SCRIPT_DIR"/*/; do
  crate_name="$(basename "$crate_dir")"
  if [[ -f "$crate_dir/Cargo.toml" ]]; then
    echo "▶  Building $crate_name..."
    wasm-pack build "$crate_dir" \
      --target web \
      --out-dir "$crate_dir/pkg" \
      --release
    echo "✓  $crate_name → $crate_dir/pkg"
  fi
done