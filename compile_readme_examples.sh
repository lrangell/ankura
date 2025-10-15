#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_PATH="$ROOT/readme_examples_profile.pkl"
OUTPUT_PATH="${1:-$ROOT/target/karabiner-readme.json}"

mkdir -p "$(dirname "$OUTPUT_PATH")"

echo "Compiling README example profile..."
(cd "$ROOT" && cargo run -- compile --config "$CONFIG_PATH" --output "$OUTPUT_PATH")
echo "✅ Generated Karabiner configuration at $OUTPUT_PATH"
