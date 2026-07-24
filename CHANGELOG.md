# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [7.0.0] - 2026-07-24

### Changed
- **Raster Pipeline Performance Overhaul**: The `image`/`video` raster-to-skeleton pipeline (Sobel edge detection, Zhang-Suen thinning) was almost entirely single-threaded despite `--threads`/rayon being available — profiling showed these two stages alone accounted for 90%+ of per-frame processing time on multi-megapixel inputs, and multithreading had no measurable effect on total runtime regardless of `--threads`. Sobel gradients now use `imageproc`'s parallel filter primitives across all cores; Zhang-Suen thinning was rewritten against a flat row-major grid (replacing nested `Vec<Vec<bool>>`) with per-iteration candidate scanning parallelized across rows. On a 6000x3375 test image this cuts single-frame processing from ~1.4s to ~0.5s on a 24-core machine (and remains correct: identical skeleton path/point counts before and after on all test images).
- **Video Frame-Level Parallelism**: The `video` subcommand previously processed frames strictly one at a time. Frames are now batched (up to `min(threads, 8)` concurrent) and processed via `std::thread::scope`, in addition to the raster pipeline's own internal parallelism. Combined with the raster pipeline rewrite, a 10s/301-frame 1920x1080 test clip (Fourier mode) went from 22.4s to 4.5s on a 24-core machine — a 4.98x improvement. `--threads 1` remains available for single-core-constrained environments, though the underlying parallel primitives carry some overhead at threads=1 (video: ~32s vs ~22s previously) — this is an intentional and expected multithreading-first tradeoff, not a regression to fix.
- **GPU FFT Pipeline Caching**: `perform_fft_gpu`/`perform_fft_batch_gpu` previously recompiled both compute pipelines (bind group layout, pipeline layout, and two `ComputePipeline`s) on every single call. These are now built once in `GpuContext` and reused. This is a correctness/efficiency fix on its own merits, but benchmarking confirms it does **not** change the GPU-vs-CPU performance picture: for Vectomancy's typical path sizes and counts, the CPU FFT path remains 2-270x faster than GPU (the bottleneck is the unavoidable per-call `device.poll(Wait)` CPU↔GPU synchronization round-trip, not pipeline setup). `--gpu` remains available and unchanged in behavior; it is not recommended for typical image/video workloads and its GPU-vs-CPU tradeoffs are unchanged by this release.

### Verification
- All raster pipeline changes preserve identical output: skeleton path counts, point counts, and rendered geometry were confirmed unchanged on all benchmark images (verified via extracted-path-count logging and visual rendering comparison).
- `cargo test --workspace` and `cargo clippy --workspace --all-targets` pass with no warnings.

## [6.4.0] - 2026-07-24

### Added
- **Video GPU/Threading Controls**: The `video` subcommand now exposes `--gpu`, `--gpu-power`, and `--threads` flags (plus matching `gpu`/`gpu_power`/`threads` keys under `[video]` in `config.toml`), mirroring the `image` subcommand. Previously these were silently ignored on `video`: the CPU thread pool was never explicitly initialized (relying on rayon's default) and there was no way to enable `wgpu` GPU acceleration for Fourier FFT batches when processing video frames.

## [6.3.1] - 2026-07-24

### Fixed
- **GIF Video Output**: `vectomancy video -o out.gif` previously accepted `.gif` as a valid output extension but always encoded frames with `libx264`, which the GIF muxer rejects (`gif muxer supports only codec gif for type video`), so every GIF export failed. Frames are now encoded with the native `gif` codec through a palette-optimized filter chain (`palettegen`/`paletteuse`).
- **Audio Detection**: The video re-encoding pipeline previously treated any existing input file as having an audio track (`args.input.exists()`), which crashed on silent inputs with `Failed to set value '1:a' for option 'map'`. Audio presence is now probed via `ffmpeg`'s stream list (`vectomancy_video::has_audio_stream`), and GIF outputs never attempt to map audio (the format doesn't support it).

## [6.3.0] - 2026-07-24

### Added
- **SVG Export Format**: New `--format svg` output for `image`, `video`, and `text` subcommands. Flattens Spline, Fourier, and Polyline/Chaikin AST curves into `<path>` elements in a standalone, viewBox-scoped SVG document. Solid colors and linear gradients are preserved via `<linearGradient>` defs.
- **CLI**: Added `svg` variant to `--format`/`-f` and automatic `.svg` extension detection for the `text` subcommand's output path.

## [6.2.0] - 2026-06-25

### Added
- **Adaptive Fourier Compression**: Dynamically determines the minimal number of Fourier terms to retain based on a cumulative energy ratio (default 99.5%).
- **CLI Flags**: Added `--fourier-adaptive` (boolean) and `--fourier-energy` (float) flags to Image and Video subcommands.
- **Config Options**: Added `fourier_adaptive` and `fourier_energy_threshold` parameters to `config.toml` under `[image]`, `[video]`, and `[text]` sections.

## [6.1.0] - 2026-06-24

### Added
- **AST Floating-Point Quantization**: Added mathematical coordinates and expression rounding (truncated to 4 decimal places by default) to optimize serialized files and rendering speed.
- **Zero Term Elimination**: In Desmos export, terms multiplied by `0.0` (e.g., `0*(t-x)`) are now omitted to keep equations as brief as possible.
- **Configuration Toggle**: Added `simplify_math` (boolean) configuration parameter under `[image]`, `[text]`, and `[video]` sections in `config.toml`.
- **CLI Flag**: Added `--no-simplify-math` to `image`, `text`, and `video` CLI subcommands to bypass rounding and retain original precision.
- **Web UI Control**: Added a "Simplify" checkbox to the Settings panel in Vectomancy Pro (Image, Playground, and Video pages), allowing users to control mathematical rounding and zero-term removal in the web browser.

### Changed
- **Default Behavior**: Spline coordinate representation is now rounded by default (saving ~40% file size and increasing browser rendering performance).
- **Template Updates**: `desmos.tera` updated to dynamically filter out `0.0` coefficients.

## [5.0.0] - 2026-06-10

### Added
- Grouped configuration under `image`, `video`, and `text` subcommands in `Config`.
- Support for `image`, `video`, and `text` subcommands in `vectomancy-cli`.
- `vectomancy-text` module for direct TTF/OTF font outline extraction.
- Automatic RAII temporary directories in integration tests.

### Fixed
- Fixed cascading configuration overrides between command-line arguments and configuration settings.
- Resolved redundant conditional compilation gates and warnings.

## [4.1.0] - 2026-05-28
### Added
- Wasm-pack targets and memory-based parsers.

## [4.0.0] - 2026-05-20
### Changed
- Replaced outdated formats (Scratch, Kmplot, Wolfram, Geogebra, Latex) with standard Spline, Fourier, and Chaikin representations.
