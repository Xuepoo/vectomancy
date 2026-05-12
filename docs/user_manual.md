# Vectomancy User Manual

Welcome to **Vectomancy**! Vectomancy is a high-performance command-line image conversion tool. It deeply parses raster images and vector files, transforming them into mathematically beautiful parametric equation collections and rendering them directly into script formats supported by major mathematical software.

## 1. Core Features

- **Multi-format Mathematical Equation Export**: Supports Python (Matplotlib), HTML5 Canvas, Desmos (`.html`), JSON, WebP, JPEG, and PNG.
- **AST Size Optimization**: Uses `Zlib + Base64` encoding to store massive floating-point matrices. This keeps the generated files compact and prevents editors and rendering engines from freezing or crashing when parsing large Abstract Syntax Trees (AST).
- **Controllable Smoothness and Rendering Modes**:
  - `--mode spline`: Reconstructs shapes with precise Bezier curve interpolation, combined with the Chaikin algorithm for smoothing to eliminate jagged, staircase-like edges.
  - `--mode fourier`: Utilizes Fourier series (based on TSP path planning) to approximate a continuous, single-stroke curve of the image.
- **Detail Level Configuration**: For math-parsing web renderers like Desmos, the `--detail` parameter (1-100) controls the level of simplification. This filters out noise paths and reduces the number of generated equations without excessive distortion, effectively avoiding rendering lag.

## 2. Quick Start

### 2.1 Installation

We provide compiled binaries for various platforms:

- **Download Precompiled Binaries**: You can download native binaries for Windows, macOS, and Linux (Debian, Arch, RedHat, openSUSE, NixOS, etc.) from the [Releases](https://github.com/Xuepoo/vectomancy/releases) page of this repository.
- **Build from Source (Rust)**:
  ```bash
  git clone https://github.com/Xuepoo/vectomancy.git
  cd vectomancy/vectomancy
  cargo build --release
  ```

### 2.2 Basic Usage

If you have an image (e.g., `assets/Tux.png`), you can convert it to a Python script with the following command:

```bash
vectomancy run assets/Tux.png --output Tux.py --format python --mode spline
```

If you want to view the mathematical rendering directly in your browser using Desmos, you can run:

```bash
vectomancy run assets/Tux.png --output Tux.html --format desmos --mode spline --chaikin-iters 2 --detail 30
```

_(Note: For complex images, it is recommended to decrease `--detail` (e.g., to 20 or 30) to significantly reduce the total number of equations and ensure the browser renders smoothly.)_

## 3. Parameter Tuning Guide (How to Get the Best Results)

Vectomancy provides several parameters that significantly affect the visual quality, file size, and smoothness of the generated vector outputs. Since CPU and GPU backend outputs are visually identical (differing only in processing speed), tuning these parameters is the key to creating beautiful mathematical art.

### 3.1 `min-path-len` (Minimum Path Length)

- **Flag**: `--min-path-len`
- **Default**: `5`
- **What it does**: Filters out paths that have fewer points than this threshold.
- **Effect when increased** (e.g., `10` - `20`): Removes small "dust" particles, compression artifacts, and tiny stray lines. Makes the final image cleaner and drastically reduces output file size. Highly recommended for noisy web images or complex backgrounds.
- **Effect when decreased** (e.g., `2` - `5`): Retains maximum detail, including stippling, fine textures, and short strokes. Best for clean, high-contrast logos.

### 3.2 `detail` (Detail Level)

- **Flag**: `--detail`
- **Default**: `50`
- **What it does**: Controls how strictly the paths are simplified before generating equations (scale of 1-100).
- **Effect when decreased** (e.g., `10` - `30`): Aggressively simplifies paths by removing points that don't deviate much from a straight line. Results in much smaller files and a more angular, low-poly aesthetic. Recommended for rendering in math software like Desmos to prevent lag.
- **Effect when increased** (e.g., `70` - `100`): Keeps almost all points, hugging the original curve precisely. Generates larger file sizes and is better for highly detailed curves where exact shapes matter.

### 3.3 `chaikin-iters` (Chaikin Smoothing Iterations)

- **Flag**: `-c`, `--chaikin-iters`
- **Default**: `0` (Off)
- **What it does**: Applies Chaikin's corner-cutting algorithm to smooth out jagged or angular paths. Only active when using `--mode chaikin`.
- **Effect when increased** (e.g., `2` - `3`): Produces very smooth, flowing, organic curves. Highly recommended for character art, anime lineart, and organic shapes to remove any "digital pixel jaggedness".
- **Effect when decreased/off** (`0` or `1`): Retains sharp corners. Essential for geometric shapes, mechanical drawings, and exact logos.

### 3.4 `mode` (Processing Algorithm)

- **Flag**: `-m`, `--mode`
- **Options**: `spline`, `fourier`, `chaikin`
- **`spline`**: Exact point-to-point drawing using Bezier/linear formulas. Great for precision.
- **`chaikin`**: Like `spline` but strictly applies smoothing. Produces the most visually pleasing, hand-drawn look for illustrations.
- **`fourier`**: Attempts to draw the entire image with a single continuous line using Fourier Epicycles (TSP approximation). Creates a unique, chaotic "single-wire" aesthetic but generates massive files and equations.

### 3.5 `terms` (Fourier Terms)

- **Flag**: `-n`, `--terms`
- **Default**: `1000`
- **What it does**: Controls the mathematical precision of the `fourier` mode.
- **Effect when increased**: The single continuous line more tightly wraps around the original image contours, preserving more detail but drastically increasing mathematical complexity.
- **Effect when decreased**: The resulting drawing becomes very loose, loopy, and abstract.

### 3.6 How to Get Vibrant Colors (Dealing with Anti-aliasing)

When extracting colors, the engine averages the pixel colors under each generated path. If your path captures too much white background or anti-aliased gray pixels (especially common when smoothing is extreme or paths are too fine), the colors will appear "washed out" or desaturated.

To achieve **vibrant, highly saturated colors**, use this recommended combination:

- **`--stroke-width 1.5` or `2.0`**: Thicker lines resist the fading effect of the renderer's anti-aliasing.
- **`-c 2` (Moderate Smoothing)**: While `3` is the smoothest, it can push paths off the original bright pixels. A value of `2` keeps the path tightly hugging the original vibrant colors while still eliminating jagged edges.
- **`--min-path-len 5` and `--tolerance 0.3`**: Filters out microscopic dust/noise that usually averages to gray, leaving only the primary colorful structures.

## 4. Configuration File & Defaults

Users can configure default settings by creating a `config.toml` in the system configuration directory.
For Linux, the path is `~/.config/vectomancy/config.toml`.

For full details on advanced global configurations, multi-file batch processing, default output behaviors, and planned GPU acceleration support, please refer to the [Configuration Guide](./configuration_guide.md).

## 5. Prerequisites and Output Usage

- **Python**: Requires `python3` and `matplotlib` (`pip install matplotlib`). Run command: `python3 output.py`.
- **LaTeX**: Requires `texlive-latexextra` or a TeX distribution with TikZ support. Compile command: `pdflatex output.tex`.
- **Wolfram**: Requires Wolfram Engine (`wolframscript`). Run command: `wolframscript -f output.txt`.
- **GeoGebra**: Open the generated `.ggb` (ZIP archive) directly in the GeoGebra application.
- **Kmplot**: Open the generated `.fkt` XML file directly in Kmplot.
- **HTML**: Open directly in modern browsers (Chrome, Firefox).

## 6. Examples

- **High-Quality Python Spline Generation**:
  `vectomancy run input.png --output out.py --format python --mode spline --chaikin-iters 2`
- **Low-Density Math Software Rendering**:
  `vectomancy run input.png --output out.ggb --format geogebra --mode spline --tolerance 2.0`

## 7. FAQ

**Q: Will my VSCode freeze when opening the generated Python or HTML files?**
**A:** No. Since version 1.0, we automatically inject anti-scanning directives (like `# pylint: disable=all` or `<!-- eslint-disable -->`) at the beginning of the generated scripts. Also, via Zlib compression, file sizes stay in the hundreds of KBs, which mainstream IDEs can open safely.

**Q: Why does GeoGebra freeze when I import the file?**

**A:** Math formula rendering software like GeoGebra is limited by internal XML tree parsing restrictions. If an image contains too much noise resulting in tens of thousands of equations, it will lag. We recommend increasing `--tolerance` (e.g., to 2.0 or 3.0) and specifying `--min-path-len` to filter out tiny noisy lines.

## 8. Container & Docker Usage

When using Vectomancy via Podman or Docker, be aware of the following:

- **Missing Output Files**: Containers run in an isolated filesystem. To access host files, you must mount a volume. For example:
  ```bash
  podman run --rm -v $(pwd):/data localhost/vectomancy:2.0.2 run /data/input.png --output /data/output.py
  ```
- **XDG_RUNTIME_DIR Warning**: If you see `error: XDG_RUNTIME_DIR is invalid or not set in the environment`, it is a harmless warning from the underlying graphics libraries attempting to query a display server in a headless container. You can silence this by setting the environment variable:
  ```bash
  podman run -e XDG_RUNTIME_DIR=/tmp/runtime-root ...
  ```
- **GPU Acceleration**: By default, processing is done on the CPU. To enable `wgpu` hardware acceleration inside a container, pass your GPU to the container and append `--gpu` to the command:
  ```bash
  podman run --device nvidia.com/gpu=all ... localhost/vectomancy:2.0.2 run --gpu ...
  ```
- **Custom Configuration**: You can map your local config file to the container:
  ```bash
  podman run -v ~/.config/vectomancy:/root/.config/vectomancy ...
  ```

---

Thank you for using Vectomancy! Enjoy the visual art brought to life by mathematical curves.
