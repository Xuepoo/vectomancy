# Vectomancy

Vectomancy is a high-performance command-line interface tool designed to parse graphic files and convert them into mathematical parametric equations and rendering scripts. It enables users to transform raster images and vector graphics into mathematical waveforms.

## Features

- **Input parsing & preprocessing:**
  - **Vector (`.svg`):** Parses paths, transforms, and basic shapes into normalized absolute coordinates.
  - **Raster (`.png`, `.jpg`, `.webp`):** Noise reduction, binarization, contour tracking, skeletonization, and point cloud reduction using the Ramer-Douglas-Peucker (RDP) algorithm.
- **Mathematical Conversion Engine:**
  - **Fourier Series Approximation (`--mode fourier`):** Uses TSP (Nearest Neighbor with 2-Opt) to find an optimal continuous path, then applies FFT to approximate the path with a configurable number of terms (`--terms`). Ideal for raster inputs or complex, non-parametric shapes.
  - **Exact Parametric Splines (`--mode spline`):** Converts SVG Bezier curves into precise parametric polynomial equation groups.
- **Template-Driven Output:** Generates outputs in various formats: LaTeX (`.tex`), Desmos HTML (`.html`), Python Matplotlib (`.py`), GeoGebra commands (`.ggb.txt`), and raw JSON (`.json`).

## Core Algorithms

The engine employs several techniques to achieve precise conversion:

- **Otsu Binarization**: Automatically determines the optimal threshold for image binarization.
- **Moore Neighborhood Tracing**: Extracts contours from binary images.
- **Ramer-Douglas-Peucker Reduction**: Simplifies paths by reducing the number of points while preserving shape.
- **TSP Nearest Neighbor + 2-Opt**: Optimizes path continuity for Fourier series approximation.
- **FFT (Fast Fourier Transform)**: Approximates paths using a configurable number of terms.

## Example Showcases

| Original Image                                     | Rendered Output                                            |
| :------------------------------------------------- | :--------------------------------------------------------- |
| ![Original Image](assets/Hatsune_miku_v2.png)      | ![Rendered Output](assets/Hatsune_miku_v2_render.png)      |
| ![Original Image](assets/Tux.png)                  | ![Rendered Output](assets/Tux_render.png)                  |
| ![Original Image](assets/Cat_November_2010-1a.jpg) | ![Rendered Output](assets/Cat_November_2010-1a_render.png) |
| ![Original Image](assets/01_khafre_north.jpg)      | ![Rendered Output](assets/01_khafre_north_render.png)      |

### Image Sources

- Miku: [https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png](https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png)
- Tux: [https://en.wikipedia.org/wiki/File:Tux.svg](https://en.wikipedia.org/wiki/File:Tux.svg)
- Cat: [https://en.wikipedia.org/wiki/Tabby_cat#/media/File:Cat_November_2010-1a.jpg](https://en.wikipedia.org/wiki/Tabby_cat#/media/File:Cat_November_2010-1a.jpg)
- Pyramid: [https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg](https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg)

## CLI Usage

```bash
vectomancy run [OPTIONS] --output <OUTPUT> <INPUT>
```

Options:

- `-o, --output <OUTPUT>`: Path for the generated output file.
- `-f, --format <FORMAT>`: Output format (python, latex, html, json, geogebra, wolfram).
- `-m, --mode <MODE>`: Conversion mode (fourier, spline).
- `-n, --terms <TERMS>`: Number of terms for Fourier approximation (default: 1000).

Configuration loads from `~/.config/vectomancy/config.toml` following the XDG Base Directory specification.

## Roadmap

- GPU acceleration via Compute Shaders (wgpu and Vulkan).
- Multi-threading improvements.
- Colored terminal output.

## License

This project is licensed under the MIT License.

## Installation

You will need the Rust toolchain installed.

```bash
cargo build --release
```
