# Vectomancy Configuration Guide

Vectomancy is designed with the **"Convention Over Configuration"** philosophy. Out of the box, it behaves intuitively with sensible defaults. However, it also offers a powerful configuration system to suit advanced workflows, tuning the mathematical engine, and selecting output platforms.

---

## Configuration Hierarchy

Vectomancy determines runtime parameters based on the following priority (from highest to lowest):

1. **CLI Arguments** (Highest Priority): Any arguments directly passed to the command line (e.g., `--mode spline`).
2. **Explicit Config File**: Settings specified in a file passed via `--config <PATH>` take precedence over global defaults.
3. **User `config.toml` Settings**: Global preferences loaded from standard XDG base directories.
4. **Internal Default Values** (Fallback): Built-in defaults (e.g., `mode = "spline"`).

### Global Configuration File Location

Vectomancy reads global preferences from a standard XDG base directory:

- **Linux:** `~/.config/vectomancy/config.toml`
- **macOS:** `~/Library/Application Support/com.vectomancy.vectomancy/config.toml`
- **Windows:** `C:\Users\%USERNAME%\AppData\Roaming\com\vectomancy\vectomancy\config\config.toml`

---

## Available Configuration Parameters

The parameters below can be defined in your `config.toml` file, passed explicitly via a custom config file, or overridden by CLI flags.

### 1. Processing Modes (`mode`)

The `mode` parameter defines which mathematical algorithm Vectomancy uses to process and map the graphical paths.

| Mode                 | Configuration Value | CLI Flag         | Description                                                                                                                                                                                                                                              |
| -------------------- | ------------------- | ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Spline (Default)** | `mode = "spline"`   | `--mode spline`  | Generates continuous mathematical parametric equations (Bezier/Polynomial splines). This mode perfectly traces curves with minimal overhead, delivering the highest visual fidelity and fastest processing speed for most line art and SVG paths.        |
| **Fourier**          | `mode = "fourier"`  | `--mode fourier` | Translates spatial coordinates into a series of frequency-domain epicycles (Sine/Cosine sums) via FFT. Used for generating "one-line drawing" animations. Note that processing dense graphics with Fourier might require tweaking the `terms` parameter. |
| **Chaikin**          | `mode = "chaikin"`  | `--mode chaikin` | Applies Chaikin's corner-cutting algorithm. This mode smoothens low-resolution polygon paths recursively. Useful when working with pixel-art or highly jagged, unaliased sketches. You can control the depth using `chaikin_iters`.                      |

### 2. Output Formats (`format`)

The `format` parameter determines the output platform and how the rendering instructions are serialized.

| Format              | Output Type             | Description                                                                                                    |
| ------------------- | ----------------------- | -------------------------------------------------------------------------------------------------------------- |
| **Image (Default)** | `.png`, `.jpg`, `.webp` | Native fast rendering. Rasterizes the calculated math data directly into an image using the internal renderer. |
| **Python**          | `.py`                   | Generates a Python script using `numpy` and `matplotlib` to render the graphic mathematically.                 |
| **Desmos**          | `.html`                 | Generates a local HTML file with an embedded Desmos Graphing API drawing the mathematical equations.           |
| **HTML**            | `.html`                 | Generates a local HTML file rendering parametric equations via Canvas API.                                     |
| **JSON**            | `.json`                 | Raw output of the parsed Vectomancy Math Abstract Syntax Tree (AST).                                           |

### 3. Math Engine Parameters

These settings allow you to fine-tune the mathematical transformations applied to the paths.

- **`terms`** (`usize`, Default: `1000`)
  - _CLI Equivalent:_ `-n, --terms`
  - _Description:_ Specifies the number of frequency terms used when running in **Fourier mode**. Higher values yield higher accuracy but slow down calculation and target rendering performance.
- **`detail`** (`u8`, Default: `50`)
  - _CLI Equivalent:_ `--detail`
  - _Description:_ A percentage-based parameter (1-100) that controls the amount of retained detail in curves (mapping internally to RDP algorithm tolerance). A lower detail aggressively simplifies paths (reducing equations), leading to faster rendering outputs in platforms like Desmos at the cost of shape accuracy.
- **`chaikin_iters`** (`usize`, Default: `None`)
  - _CLI Equivalent:_ `-c, --chaikin-iters`
  - _Description:_ Used specifically for `mode = "chaikin"`. Determines the recursive depth of the corner-cutting algorithm. More iterations result in smoother curves.
- **`min_path_len`** (`usize`, Default: `5`)
  - _CLI Equivalent:_ `--min-path-len`
  - _Description:_ Discards extracted paths (from raster parsing) that contain fewer points than this threshold. Excellent for removing small pixel noise, dust, or artifacts from scanned images.

### 4. Visuals & Image Rendering

Used when exporting native `Image` formats or embedding color data to Python/Html targets.

- **`color`** (`bool`, Default: `false`)
  - _CLI Equivalent:_ `--color`
  - _Description:_ Enables color sampling from the original image (instead of strictly monochrome lines). Target paths will inherit the localized RGB averages.
- **`bg_transparent`** (`bool`, Default: `false`)
  - _CLI Equivalent:_ `--bg-transparent`
  - _Description:_ For native image output, defines whether the canvas background should be completely transparent (Alpha 0) instead of solid white.
- **`stroke_width`** (`f32`, Default: `1.0`)
  - _CLI Equivalent:_ `--stroke-width`
  - _Description:_ Sets the line thickness for native image rendering.
- **`width` / `height`** (`u32`, Default: `None`)
  - _CLI Equivalent:_ `--width`, `--height`
  - _Description:_ Overrides the target dimension for the output. If left unspecified, defaults to the parsed dimensions of the source file.
- **`bit_depth`** (`u8`, Default: `None`)
  - _CLI Equivalent:_ `--bit-depth`
  - _Description:_ Changes the output image's channel depth (e.g., 8, 16).
- **`color_space`** (`String`, Default: `None`)
  - _CLI Equivalent:_ `--color-space`
  - _Description:_ Forces a specific color space (`sRGB`, `DisplayP3`, `CMYK`).

### 5. Hardware Acceleration & Performance

Vectomancy allows processing through multi-core CPUs (`rayon`) or GPU Compute Shaders (`wgpu`).

- **`threads`** (`usize`, Default: `1`)
  - _CLI Equivalent:_ `--threads`
  - _Description:_ Defines the number of CPU threads allocated for workload parallelization. Setting this to >1 scales processing significantly when parsing vast amounts of paths.
- **`gpu`** (`bool`, Default: `false`)
  - _CLI Equivalent:_ `--gpu`
  - _Description:_ Forces the math engine (like the FFT transform) to be offloaded to the GPU using WebGPU Compute Shaders.
- **`gpu_power`** (`String`, Default: `None`)
  - _CLI Equivalent:_ `--gpu-power`
  - _Description:_ Informs WGPU which adapter to prioritize (`HighPerformance` for Dedicated GPU, `LowPower` for Integrated).

---

## ⚡ Why is `--gpu` sometimes slower than CPU?

You might notice that setting `gpu = true` in your config doesn't always make rendering faster. In fact, for many simple SVGs and low-resolution raster images, **CPU multithreading is significantly faster**.

Why does this happen?

1. **PCIe Bus Transfer Overhead**: The fundamental nature of Vector/Math rendering requires extracting a massive amount of mathematically connected vertices (nodes). Transferring these thousands of discrete geometry nodes across the PCI-Express bus to the GPU VRAM takes time.
2. **WGPU Context Initialization**: Initializing the Vulkan/Metal/DX12 pipelines, compiling the shaders, and allocating GPU memory buffers inherently introduces a fixed 100ms - 300ms latency.
3. **CPU vs. GPU Workloads**: GPUs excel at doing the _same_ operation on massive grids of pixels in parallel. However, calculating Splines, tracing TSP paths, and simplifying graphs are highly _sequential_ algebraic tasks. Vectomancy's CPU backend leverages `rayon` to distribute these calculations perfectly across high-performance CPU cores.

**When should you use `gpu = true`?**
Enable GPU acceleration only when processing **extremely dense point arrays** (e.g., executing a Fourier transform requiring `terms` > 10,000 on a giant high-res canvas).

---

### Complete Example `config.toml`

```toml
# Default Output & Rendering Mode
mode = "spline"
format = "python"
default_output_dir = "/home/user/Pictures/VectomancyOut"

# Mathematical Tweaks
terms = 2000
tolerance = 0.5
min_path_len = 10
chaikin_iters = 3

# Visuals & Color
color = true
bg_transparent = true
stroke_width = 1.5

# Advanced Image Formats
bit_depth = 16
color_space = "sRGB"

# Hardware Acceleration & Performance
gpu = false
threads = 8
gpu_power = "HighPerformance"
```
