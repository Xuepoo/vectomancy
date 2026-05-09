# Vectomancy User Manual

Welcome to **Vectomancy**! Vectomancy is a high-performance command-line image conversion tool. It deeply parses raster images and vector files, transforming them into mathematically beautiful parametric equation collections and rendering them directly into script formats supported by major mathematical software.

## 1. Core Features

- **Multi-format Mathematical Equation Export**: Supports Python (Matplotlib), LaTeX (TikZ), Wolfram, GeoGebra (`.ggb`), Kmplot (`.fkt`), HTML5 Canvas, and native JSON.
- **AST Size Optimization**: Uses `Zlib + Base64` encoding to store massive floating-point matrices. This keeps the generated files compact and prevents editors and rendering engines from freezing or crashing when parsing large Abstract Syntax Trees (AST).
- **Controllable Smoothness and Rendering Modes**:
  - `--mode spline`: Reconstructs shapes with precise Bezier curve interpolation, combined with the Chaikin algorithm for smoothing to eliminate jagged, staircase-like edges.
  - `--mode fourier`: Utilizes Fourier series (based on TSP path planning) to approximate a continuous, single-stroke curve of the image.
- **Lightweight & Tolerance Configuration**: For pure math-parsing software like GeoGebra/Kmplot, `--tolerance` (RDP algorithm tolerance) and `--min-path-len` parameters are provided. This filters out noise paths and drastically reduces the number of generated equations without excessive distortion, effectively avoiding rendering lag.

## 2. Quick Start

### 2.1 Installation

We provide compiled binaries for various platforms:

- **Download Precompiled Binaries**: You can download native binaries for Windows, macOS, and Linux (Debian, Arch, RedHat, openSUSE, NixOS, etc.) from the [Releases](https://github.com/Xuepoo/vectomanct/releases) page of this repository.
- **Build from Source (Rust)**:
  ```bash
  git clone https://github.com/Xuepoo/vectomanct.git
  cd vectomanct/vectomancy
  cargo build --release
  ```

### 2.2 Basic Usage

If you have an image (e.g., `assets/Tux.png`), you can convert it to a Python script with the following command:

```bash
./vectomancy run assets/Tux.png --output Tux.py --format python --mode spline
```

If you want to double-click and directly open the rendered equations in mathematical software (like GeoGebra), you can run:

```bash
./vectomancy run assets/Tux.png --output Tux.ggb --format geogebra --mode spline --chaikin-iters 2 --tolerance 2.0
```

_(Note: For GeoGebra, it is recommended to increase the `--tolerance`, such as 2.0, to significantly reduce the total number of equations and ensure the software runs smoothly.)_

## 3. Advanced Configuration Parameters

You can use `vectomancy --help` to see all parameters. Common ones include:

- `--format <FORMAT>`: Output file format. Options: `python`, `latex`, `html`, `json`, `geogebra`, `wolfram`, `kmplot`.
- `--mode <MODE>`: Calculation mode. Options: `spline` (Bezier curve equations, recommended for exact displays), `fourier` (Fourier series single-stroke approximation).
- `--chaikin-iters <N>`: The number of Chaikin smoothing iterations applied in Spline mode. Default is `0`. Higher values create smoother corners. Recommended settings are `1` or `2`.
- `--tolerance <FLOAT>`: Ramer-Douglas-Peucker algorithm simplification tolerance. Larger values omit more vertices and equations. When rendering large images to mathematical software, `2.0` is recommended.
- `--min-path-len <FLOAT>`: Ignores noise paths with a total length below this value. Increasing this value removes tiny speckles during the image conversion process.

## 4. Configuration File

Users can configure default settings by creating a `config.toml` in the system configuration directory.
For Linux, the path is `~/.config/vectomancy/config.toml`.

Example `config.toml`:

```toml
chaikin_iters = 2
tolerance = 1.5
min_path_len = 5.0
```

## 5. Prerequisites and Output Usage

- **Python**: Requires `python3` and `matplotlib` (`pip install matplotlib`). Run command: `python3 output.py`.
- **LaTeX**: Requires `texlive-latexextra` or a TeX distribution with TikZ support. Compile command: `pdflatex output.tex`.
- **Wolfram**: Requires Wolfram Engine (`wolframscript`). Run command: `wolframscript -f output.txt`.
- **GeoGebra**: Open the generated `.ggb` (ZIP archive) directly in the GeoGebra application.
- **Kmplot**: Open the generated `.fkt` XML file directly in Kmplot.
- **HTML**: Open directly in modern browsers (Chrome, Firefox).

## 6. Examples

- **High-Quality Python Spline Generation**:
  `./vectomancy run input.png --output out.py --format python --mode spline --chaikin-iters 2`
- **Low-Density Math Software Rendering**:
  `./vectomancy run input.png --output out.ggb --format geogebra --mode spline --tolerance 2.0`

## 7. FAQ

**Q: Will my VSCode freeze when opening the generated Python or HTML files?**
**A:** No. Since version 1.0, we automatically inject anti-scanning directives (like `# pylint: disable=all` or `<!-- eslint-disable -->`) at the beginning of the generated scripts. Also, via Zlib compression, file sizes stay in the hundreds of KBs, which mainstream IDEs can open safely.

**Q: Why does GeoGebra freeze when I import the file?**
**A:** Math formula rendering software like GeoGebra is limited by internal XML tree parsing restrictions. If an image contains too much noise resulting in tens of thousands of equations, it will lag. We recommend increasing `--tolerance` (e.g., to 2.0 or 3.0) and specifying `--min-path-len` to filter out tiny noisy lines.

---

Thank you for using Vectomancy! Enjoy the visual art brought to life by mathematical curves.
