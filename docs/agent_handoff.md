# Vectomancy Agent Handoff & Roadmap

**Target Audience:** Future Opencode Agents & Maintainers
**Current Version:** v2.0.1

Welcome to the Vectomancy project! You are inheriting a highly optimized, hardware-accelerated mathematical parser and renderer written in Rust.

## 1. System Overview & Core Strengths

- **The Core Engine**: Vectomancy transforms raster/vector images into parametric equations and AST graphs.
- **Math Limits Pushed**: We use `kiddo` (KD-Tree) for O(N log N) Traveling Salesperson pathing, `rustfft` mapped with memory pools, and `rayon` for deep multi-core processing.
- **GPU Rasterization**: We recently dropped `tiny-skia` for `wgpu` + `lyon_tessellation`. The engine natively speaks WebGPU/Vulkan/Metal/DX12 to render generated math paths instantly into PNGs.

## 2. Future Roadmap: GUI Evolution

Currently, Vectomancy is a highly-capable CLI tool tailored for geeks. Your primary mission is to build user-friendly interfaces around this core.

### Phase A: The Zero-Cost Web App (WASM + WebGPU)

- **Goal**: Create a web interface hosted purely statically on Cloudflare Pages, Vercel, or GitHub Pages.
- **Architecture**: Compile the Rust core to WebAssembly (`wasm32-unknown-unknown`).
- **Rendering**: The existing `wgpu` implementation is perfectly suited for WebGPU APIs in the browser. You just need to pipe the `wgpu::Surface` to an HTML5 `<canvas>`.
- **Challenges**:
  - File I/O must shift from local `std::fs` to memory buffers (`&[u8]`).
  - `rayon` threading will need `wasm-bindgen-rayon` and `SharedArrayBuffer` HTTP headers.
  - Remove/Isolate `directories` crate caching for the WASM target using `#[cfg(not(target_arch = "wasm32"))]`.

### Phase B: Desktop Native (Tauri)

- **Goal**: A zero-compromise desktop app that does not require an internet connection.
- **Architecture**: Wrap the core inside Tauri. This gives us a web-based frontend (HTML/CSS/JS) but runs the math engine directly on the OS native hardware.
- **Why?**: WASM can incur a 10-20% performance overhead. Tauri runs at 100% native CPU/GPU speed and provides full filesystem access.

### Phase C: Mobile Platforms

- Consider expanding the Tauri or React Native wrappers to deploy on iOS/Android app markets.

## 3. Distribution & Package Management

To reach more developers before the GUI is fully ready, manage the CLI tool's deployment on popular package managers:

- **Arch Linux**: Create an AUR package (`PKGBUILD`).
- **macOS**: Setup a Homebrew Tap.
- **Windows**: Add a Scoop manifest.

## 4. Containerization Notes (Docker / Podman)

A `Dockerfile` is included in the project root to ensure clean, isolated builds.

- **Multi-Stage Build**: We use `rust:slim-bookworm` for building, and `debian:bookworm-slim` for the runtime.
- **GPU Passthrough**: Note that `wgpu` relies on Vulkan/Mesa drivers inside the container. To leverage hardware acceleration via Podman/Docker, users must map the GPU devices (e.g., `--device /dev/dri` on Linux) to the container.
- **Data Mounts**: The container defaults to `/data` as the working directory. Instruct users to mount their images to `/data`.

## 5. Strict Constraints

- **Keep it clean**: Do not mix CLI logic with core math or emitter logic.
- **XDG Base Dirs**: Stick to the `directories` crate for local app data. Never hardcode `$HOME`.
- **Language**: All documentation, comments, and commit messages MUST be in English.

Good luck, and keep pushing the performance envelope!
