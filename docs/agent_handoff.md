# Vectomancy Project: AI Agent Handoff & Architecture Manifest

**Target Audience:** Future Gemini / Claude / Opencode Agents & Human Maintainers
**Current Version:** v4.1.0 (Web GUI Integrated)
**Last Updated:** 2026-05-20

Welcome to the Vectomancy project! You are inheriting a highly optimized, hardware-accelerated mathematical parser, renderer, and web application. This document serves as the absolute source of truth for repository structure, architecture algorithms, operations, and strict agent constraints.

---

## 1. Project Overview & Directory Structure

The project is a monorepo containing the core Rust CLI and a fully static WebAssembly + Zola frontend.

```text
/mnt/data/Workspace/Projects/vectomancy/
├── vectomancy/                  # 🦀 Core Rust Engine & CLI Tool
│   ├── src/math/                # Core algorithms (Spline, Fourier, Chaikin, TSP solver)
│   ├── src/models/              # AST and vector representations
│   ├── src/emitter/             # Output generators (SVG, JSON, HTML)
│   └── Cargo.toml               # Publishes to crates.io
├── vectomancy-web/              # 🌐 Web UI & WASM Integration
│   ├── wasm-engine/             # Rust crate bridging `vectomancy` to `wasm-bindgen`
│   └── zola-site/               # Zola Static Site Generator (HTML/CSS/JS)
├── docs/                        # Project documentation & this handoff file
├── packaging/                   # Distribution packages (e.g., AUR PKGBUILDs)
├── tmp/                         # ⚠️ TEMPORARY DIR (Required for all AI workspace actions)
└── .github/workflows/           # CI/CD Pipelines
```

---

## 2. Core Architecture & Algorithms

### 2.1 The Math Engine
Vectomancy transforms raster/vector images into pure parametric equations.
- **Zhang-Suen Thinning**: Used to extract single-pixel skeletons from bitmaps.
- **TSP Pathing (KD-Tree)**: Uses `kiddo` for O(N log N) spatial sorting to connect discrete pixels into continuous paths.
- **WASM Memory Parsing**: The core has been refactored to support `parse_memory(&[u8])`, allowing zero-copy or minimal-copy operations directly from JavaScript `Uint8Array`s without relying on `std::fs`.

### 2.2 Mathematical Emitting Modes
1. **Spline (Bezier)**: High-precision curve fitting. Best for lineart and logos.
2. **Fourier (Epicycle)**: Uses `rustfft` to convert points into frequency domain rotating vectors. Creates chaotic, single-wire "string art" aesthetics.
3. **Chaikin (Polyline)**: Smooths discrete paths using corner-cutting iterative algorithms.

---

## 3. Web Frontend Architecture (`vectomancy-web`)

The Web interface is fully static, extremely fast, and completely decoupled from external render-blocking dependencies.

### 3.1 Tech Stack
- **Framework**: Hybrid Zola (SSG) + Vanilla JS / WebAssembly.
- **Styling**: `nes.min.css` (8-bit retro aesthetic) + Vanilla CSS. *No Tailwind*.
- **WASM Bridge**: `wasm-pack` with `serde-wasm-bindgen` transforms Rust AST into native JS objects. `rayon` and `wgpu` are deliberately disabled/mocked in the WASM target to prevent browser compatibility issues.

### 3.2 "Offscreen & Snapshot" Rendering (Canvas)
To prevent DOM explosion when rendering tens of thousands of Fourier/Spline vectors, the application uses an offscreen rendering pattern. Calculations yield vectors, which are drawn to a `OffscreenCanvas`. This is then converted to an `ImageBitmap` (Snapshot) and painted to the main DOM `<canvas>`. This ensures pan/zoom operations remain locked at 60fps regardless of math complexity.

### 3.3 Extreme Performance & SEO (Lighthouse 97~100)
- **Asset CDN**: R2 bucket is bound to a custom domain (`https://cdn.xuepoo.xyz`) for Edge Caching.
- **Self-Hosted Assets**: `Press Start 2P` font (`.woff2`) and `nes.min.css` are hosted locally in `/static/`. NO Google Fonts or unpkg external links are allowed.
- **Thumbnails**: Gallery images are highly compressed WebP (`-thumb.webp`, ~max 400px height, 50% quality) with explicit `width` and `height` attributes to prevent CLS (Cumulative Layout Shift).
- **SEO Elements**: `zola.toml` is configured to auto-generate `sitemap.xml` and `robots.txt`. The `base_url` is strictly set to `https://vectomancy.xuepoo.xyz`.
- **Preloading**: Critical assets (WASM binary, local fonts, local CSS) utilize `<link rel="preload">`.

---

## 4. DevOps, Deployment & Infrastructure

### 4.1 Cloudflare Pages (Frontend)
- **Deployment Command**:
  ```bash
  cd vectomancy-web/zola-site
  zola build
  wrangler pages deploy public --project-name vectomancy --branch main
  ```
- **Crucial Rule**: Cloudflare Pages ONLY recognizes the `main` or `master` branch as "Production". If you deploy from `feat/wasm-engine` without specifying `--branch main`, it will generate a `Preview` deployment which is penalized by an `X-Robots-Tag: noindex` header, destroying SEO. **Always deploy to `main` for production updates.**

### 4.2 Cloudflare R2 (Assets)
- **Upload Command**:
  ```bash
  wrangler r2 object put "vectomancy-assets/path/to/file" --file "./local/file" --remote
  ```
- **Domain**: R2 is served via `https://cdn.xuepoo.xyz`. Do NOT use the default `*.r2.dev` domains in HTML templates, as they bypass CDN caching.

### 4.3 Distribution & Packaging
- **AUR (Arch Linux)**: PKGBUILDs exist in `packaging/aur/vectomancy` (builds from source) and `packaging/aur/vectomancy-bin` (pulls pre-compiled binaries from GitHub Releases).
- **Crates.io**: The core `vectomancy` crate is published via `cargo publish`.

---

## 5. Strict Operational Constraints for AI Agents

**⚠️ YOU MUST ADHERE TO THESE RULES DURING YOUR OPERATION:**

1. **Working Directory Cleanliness**: NEVER create temporary scripts, test images, or `.log` files in the repository root or inside `.git`. ALWAYS use `/mnt/data/Workspace/Projects/vectomancy/tmp/` (or the relative `tmp/` dir mapped by your environment) for scratchpads, downloads, and format conversions.
2. **Network/Proxy Requirements**: The local environment resides behind a proxy. If `curl`, `wrangler`, or `cargo` fail due to timeouts, ensure environment variables are set: `export http_proxy="http://127.0.0.1:1080"` and `https_proxy="http://127.0.0.1:1080"`.
3. **Pre-commit Hooks**: The repository enforces strict formatting via `pre-commit`. Before pushing code, ensure hooks pass. If they fail (e.g., trailing whitespace, EOF fixes), git will automatically fix them. Re-stage (`git add .`) and re-commit.
4. **Architectural Integrity**: Do not bypass Rust's type system or use JavaScript `any` casts loosely. Maintain rigorous `serde` boundaries between WASM and JS.
5. **No regressions**: When adding UI elements to the Zola site, ensure `aria-label` attributes are included and check that color contrast ratios remain compliant. Run `lighthouse` audits natively via headless chrome after significant UI changes.

Good luck, and maintain the performance envelope!
