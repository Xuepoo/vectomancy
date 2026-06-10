# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
