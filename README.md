# landolt-sim
ランドルト環の網膜像シミュレーション

A dynamic Landolt C retinal image simulator built with **Rust + WebAssembly** for computation and **React** for the UI.

## What it does

- Renders a Landolt C optotype (visual acuity test ring) on an HTML canvas
- Simulates the retinal image by applying a Gaussian PSF (point spread function) convolution
- All parameters are adjustable in real time via sliders:
  - **Outer radius** – size of the ring in pixels
  - **Ring width ratio** – thickness relative to outer radius
  - **Gap angle** – angular size of the opening (°)
  - **Rotation** – clockwise rotation of the gap from the top (°)
  - **PSF σ** – blur amount simulating optical degradation (pixels)

## Tech stack

| Layer | Technology |
|-------|-----------|
| Simulation core | Rust → WebAssembly (via [wasm-pack](https://rustwasm.github.io/wasm-pack/)) |
| UI | React 19 + TypeScript (via [Vite](https://vitejs.dev/)) |

## Building

```bash
# One-shot build (requires Rust, wasm-pack, Node.js)
./build.sh
```

Or step by step:

```bash
# 1. Compile Rust → Wasm
cd landolt-wasm
wasm-pack build --target web --out-dir ../frontend/src/wasm

# 2. Build React frontend
cd ../frontend
npm ci
npm run build
# Output in frontend/dist/
```

## Development

```bash
# Build wasm first (only needed once, or after Rust changes)
cd landolt-wasm && wasm-pack build --target web --out-dir ../frontend/src/wasm && cd ..

# Start Vite dev server with HMR
cd frontend && npm run dev
```

Open <http://localhost:5173> in your browser.
