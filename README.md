# Vectomancy (矢量魔法)

Vectomancy is a high-performance command-line interface (CLI) tool designed to deeply parse various graphic files (both raster and vector) and convert them into mathematical parametric equations and executable rendering scripts. This allows seamless display in multi-language and multi-platform graphics engines.

## Features

- **Input parsing & preprocessing:**
  - **Vector (`.svg`):** Parses paths, transforms, and basic shapes into normalized absolute coordinates.
  - **Raster (`.png`, `.jpg`, `.webp`):** Noise reduction, binarization, contour tracking, skeletonization, and point cloud reduction using the Ramer-Douglas-Peucker (RDP) algorithm.
- **Mathematical Conversion Engine:**
  - **Fourier Series Approximation (`--mode fourier`):** Uses TSP (Nearest Neighbor with 2-Opt) to find an optimal continuous path, then applies FFT to approximate the path with a configurable number of terms (`--terms`). Ideal for raster inputs or complex, non-parametric shapes.
  - **Exact Parametric Splines (`--mode spline`):** Converts SVG Bezier curves into precise parametric polynomial equation groups.
- **Template-Driven Output:** Generates outputs in various formats: LaTeX (`.tex`), Desmos HTML (`.html`), Python Matplotlib (`.py`), GeoGebra commands (`.ggb.txt`), and raw JSON (`.json`).

## Installation

You will need the Rust toolchain installed.

```bash
cargo build --release
```

## Usage

Vectomancy operates through the `run` command. 

### Fourier Mode (Raster Images)

Best used for raster graphics. The tool traces contours, reduces points via RDP, solves TSP to form a path, and performs an FFT.

```bash
cargo run --release -- run input.png --mode fourier --terms 1000 --format python --output output.py
```

### Spline Mode (Vector Images)

Best used for exact mapping of SVGs. This mode translates SVG paths directly into $t$-parameterized polynomials.

```bash
cargo run --release -- run input.svg --mode spline --format latex --output output.tex
```

## Mathematical Background

### Fourier Pipeline (RDP -> TSP -> FFT)
1. **RDP (Ramer-Douglas-Peucker):** Reduces the number of points extracted from the image contour by downsampling points that lie close to line segments.
2. **TSP (Traveling Salesperson Problem):** Orders the points into a single continuous path suitable for 1D signal analysis.
3. **FFT (Fast Fourier Transform):** Treats the 2D path as complex numbers and performs an FFT to extract frequency components, generating epicycle terms.

### Spline Pipeline
SVG paths consist of Line, Quadratic Bezier, and Cubic Bezier segments. Vectomancy maps these directly to parametric polynomials:
- $x(t) = at^3 + bt^2 + ct + d$
- $y(t) = et^3 + ft^2 + gt + h$
for $t \in [0, 1]$.

## Architecture
Vectomancy uses a Hexagonal Architecture (Ports and Adapters) with a decoupled math engine and I/O handlers. It complies with the XDG Base Directory specification for configuration.

## License
MIT
