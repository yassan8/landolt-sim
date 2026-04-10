#!/usr/bin/env bash
# Build script: compile Rust/Wasm, then build React frontend
set -euo pipefail

echo "==> Building Rust/Wasm..."
(cd landolt-wasm && wasm-pack build --target web --out-dir ../frontend/src/wasm)

echo "==> Installing frontend dependencies..."
(cd frontend && npm ci)

echo "==> Building frontend..."
(cd frontend && npm run build)

echo "==> Done! Output is in frontend/dist/"
