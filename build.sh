#!/usr/bin/env bash
# build.sh — Build both WASM crates with wasm-pack
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"

echo "→ [1/2] Building step-renderer..."
(cd "$ROOT/crates/step-renderer" && \
  wasm-pack build --target web --out-dir "$ROOT/pkg/step_renderer" && \
  wasm-pack build --target bundler --out-dir "$ROOT/pkg/step_renderer_bundler")


echo "→ [2/2] Building lecture-assembler..."
(cd "$ROOT/crates/lecture-assembler" && \
  wasm-pack build --target web --out-dir "$ROOT/pkg/lecture_assembler" && \
  wasm-pack build --target bundler --out-dir "$ROOT/pkg/lecture_assembler_bundler")

echo ""
echo "✓ Build complete."
echo "  Serve with:  python3 -m http.server 8080"
echo "  Then open:   http://localhost:8080"
