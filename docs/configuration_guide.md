# Configuration Guide

Vectomancy is designed with the **"Convention Over Configuration"** philosophy. Out of the box, it behaves intuitively with sensible defaults. However, it also offers a powerful configuration system to suit advanced workflows.

## Configuration Hierarchy

Vectomancy determines runtime parameters based on the following priority:

1. **CLI Arguments** (Highest Priority)
2. **`config.toml` Settings** (Global User Preferences)
3. **Internal Default Values** (Fallback)

## Global Configuration File (`config.toml`)

Vectomancy reads global preferences from a standard XDG base directory.

- **Linux:** `~/.config/vectomancy/config.toml`
- **macOS:** `~/Library/Application Support/com.vectomancy.vectomancy/config.toml`
- **Windows:** `C:\Users\%USERNAME%\AppData\Roaming\com\vectomancy\vectomancy\config\config.toml`

### Example `config.toml`

```toml
# Default Output & Rendering Mode
mode = "spline"
format = "png"
default_output_dir = "/home/user/Pictures/VectomancyOut"

# Visuals & Color
color = true
bg_transparent = true
stroke_width = 1.5

# Advanced Image Formats
bit_depth = 16
color_space = "sRGB"

# Hardware Acceleration (Experimental, planned for next version)
gpu_acceleration = false

# Curve Fitting & Smoothing Defaults
tolerance = 0.5
min_path_len = 5
chaikin_iters = 0
terms = 1000
```

## Detailed Parameter Reference

### 🛠️ Core Execution Logic

- **Output Format (`format`)**: Defines the target rendering output. Defaults to `png`. Other options include `python`, `geogebra`, `json`, `html`, `wolfram`, `latex`, `kmplot`, `jpg`, `webp`.
- **Rendering Mode (`mode`)**:
  - `spline` (Default): Uses continuous B-spline equations for high-fidelity curve rendering.
  - `chaikin`: Polyline-based rendering utilizing Chaikin's corner-cutting subdivision.
  - `fourier`: Approximates closed shapes using epicycles.
- **Default Output Directory (`default_output_dir`)**: If set, batch-processed files and outputs without `--output` will automatically drop into this folder. If empty, Vectomancy writes to the current working directory (`./`).

### 🎨 Visuals and Color Profiles

- **Colored Drawing (`color`)**: Set to `true` to enable colored stroke sampling. (Default: `false`, renders black strokes).
- **Transparent Background (`bg_transparent`)**: If `true`, pure image formats like PNG will contain an Alpha channel. (Default: `false`, rendering over a white canvas).
- **Stroke Width (`stroke_width`)**: Float value defining the thickness of the paths. (Default: `1.0`).
- **Bit Depth (`bit_depth`)**: (8, 10, 16, 32). Useful for HDR pipelines. Defaults to 8-bit.
- **Color Space (`color_space`)**: Determines the color gamut. Currently defaults to `sRGB`. (`DisplayP3` and `CMYK` support are stubs for future expansion).

### 📐 Math & Engine Tuning

- **RDP Tolerance (`tolerance`)**: Ramer-Douglas-Peucker simplification tolerance. Lower values = closer to original pixels, larger files. Higher values = smoother paths, fewer equations. (Default: `0.5`).
- **Minimum Path Length (`min_path_len`)**: Throws out microscopic artifacts and tiny squiggles. (Default: `5` pixels/points).
- **Chaikin Smoothing Iterations (`chaikin_iters`)**: Applies smoothing to the skeleton paths. Useful when preprocessing for Splines. (Default: `0`).
- **Fourier Terms (`terms`)**: The number of rotating circles/epicycles used. Only applies to `fourier` mode. (Default: `1000`).

### ⚡ Hardware Acceleration (Future Roadmap)

- **GPU Acceleration (`gpu_acceleration`)**: Currently a `false` stub. In future major releases, setting this to `true` will offload raster operations, path thinning, and rendering to the GPU for massive concurrency.
