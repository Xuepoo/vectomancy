# Vectomancy

Vectomancy is a high-performance command-line interface (CLI) tool designed to deeply parse graphic files (raster and vector) and convert them into mathematical parametric equations and executable rendering scripts.

## Core Features

- **Polymorphic Input Processing**: 
  - Raster Mode: Reads PNG, JPG, WEBP. Performs grayscale conversion, binarization, edge tracking, and point cloud reduction (RDP).
  - Vector Mode: (Planned) Parses SVG paths and transforms them into normalized absolute coordinates.
- **Mathematical Engine**:
  - Fourier Approximation: Uses a Nearest Neighbor Traveling Salesperson Problem (TSP) solver to order discrete points, then performs Fast Fourier Transform (FFT) to generate parametric equations.
- **Template-Driven Output**:
  - Uses `tera` templates to generate outputs in various formats (Python Matplotlib, LaTeX, etc.).

## Installation

Ensure you have the Rust toolchain installed.

```bash
git clone <repository_url>
cd vectomancy/vectomancy
cargo build --release
```

## Usage

```bash
vectomancy run <INPUT_FILE> [OPTIONS]
```

### Options

- `-o, --output <OUTPUT>`: Output file path.
- `-f, --format <FORMAT>`: Output format (`python`, `latex`, `html`, `json`). Default is `python`.
- `-m, --mode <MODE>`: Processing mode (`fourier`, `spline`).
- `-n, --terms <TERMS>`: Number of Fourier terms to use. Default is `1000`.
- `-v, --verbose`: Enable verbose logging.

### Example

Convert a raster image to a Python matplotlib script using 500 Fourier terms:

```bash
vectomancy run input.png --output output.py --format python --terms 500 --verbose
```

## Development Workflow

This project adheres to Clean Code and Hexagonal Architecture principles. The core execution pipeline flows through:

1. **Initialization**: CLI parsing via `clap`.
2. **Parser**: Polymorphic input handling (`src/parser/`).
3. **Math Engine**: TSP ordering and FFT calculations (`src/math/`).
4. **Emitter**: Tera template rendering (`src/emitter/`).

## License

MIT License
