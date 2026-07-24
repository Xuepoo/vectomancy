# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
