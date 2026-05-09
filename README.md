# Vectomancy

[English](README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [Français](README.fr.md) | [Español](README.es.md)

Vectomancy is a high-performance command-line interface tool designed to parse graphic files and convert them into mathematical parametric equations and rendering scripts. It enables users to transform raster images and vector graphics into mathematically beautiful waveforms.

## Example Showcases

| Original Image                                | Rendered Output                                       |
| :-------------------------------------------- | :---------------------------------------------------- |
| ![Original Image](assets/dolphin.jpg)         | ![Rendered Output](assets/dolphin_render.png)         |
| ![Original Image](assets/Hatsune_miku_v2.png) | ![Rendered Output](assets/Hatsune_miku_v2_render.png) |
| ![Original Image](assets/Tux.png)             | ![Rendered Output](assets/Tux_render.png)             |
| ![Original Image](assets/01_khafre_north.jpg) | ![Rendered Output](assets/01_khafre_north_render.png) |

### Image Sources

- Dolphin: [https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg](https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg)
- Miku: [https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png](https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png)
- Tux: [https://en.wikipedia.org/wiki/File:Tux.svg](https://en.wikipedia.org/wiki/File:Tux.svg)
- Pyramid: [https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg](https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg)

## Features

- **Multi-format Mathematical Equation Export**: Supports Python (Matplotlib), LaTeX (TikZ), Wolfram, GeoGebra (`.ggb`), Kmplot (`.fkt`), HTML5 Canvas, and native JSON.
- **AST Size Optimization**: Uses `Zlib + Base64` encoding to store massive floating-point matrices. This keeps the generated files compact and prevents editors and rendering engines from freezing or crashing when parsing large files.
- **Controllable Smoothness and Rendering Modes**:
  - `--mode spline`: Reconstructs shapes with precise Bezier curve interpolation, combined with the Chaikin algorithm for smoothing to eliminate jagged, staircase-like edges.
  - `--mode fourier`: Utilizes Fourier series (based on TSP path planning) to approximate a continuous, single-stroke curve of the image.

For a deeper dive into the mathematical algorithms (like Otsu Binarization, Ramer-Douglas-Peucker reduction, Moore Neighborhood Tracing, and FFT), please refer to the [User Manual](docs/user_manual.md).

## Installation

You will need the Rust toolchain installed to build from source.

```bash
git clone https://github.com/Xuepoo/vectomancy.git
cd vectomancy/vectomancy
cargo build --release
```

Precompiled binaries for Linux (Debian, Arch, RedHat, openSUSE, NixOS), Windows, and macOS are available in the [GitHub Releases](https://github.com/Xuepoo/vectomancy/releases).

## CLI Usage

```bash
./target/release/vectomancy run [OPTIONS] --output <OUTPUT> <INPUT>
```

Options:

- `-o, --output <OUTPUT>`: Path for the generated output file.
- `-f, --format <FORMAT>`: Output format (python, latex, html, json, geogebra, wolfram, kmplot).
- `-m, --mode <MODE>`: Conversion mode (fourier, spline).
- `-n, --terms <TERMS>`: Number of terms for Fourier approximation (default: 1000).

Configuration loads from `~/.config/vectomancy/config.toml` following the XDG Base Directory specification.

## FAQ

**Q: Will my VSCode freeze when opening the generated Python or HTML files?**
**A:** No. We automatically inject anti-scanning directives (like `# pylint: disable=all` or `<!-- eslint-disable -->`) at the beginning of the generated scripts. Via Zlib compression, file sizes stay small, which mainstream IDEs can open safely.

**Q: Why does GeoGebra freeze when I import the file?**
**A:** Math formula rendering software is limited by internal XML tree parsing restrictions. If an image contains too much noise resulting in tens of thousands of equations, it will lag. We recommend increasing `--tolerance` (e.g., to 2.0 or 3.0) and specifying `--min-path-len` to filter out tiny noisy lines. See the [User Manual](docs/user_manual.md) for detailed tuning options.

## License

This project is licensed under the MIT License.
