---
name: vectomancy-cli
description: "Vectomancy CLI: convert images/videos to parametric equations (Spline, Fourier, Chaikin)."
tags: [rust, cli, image-processing, math, bezier, fourier, chaikin, video]
triggers:
  - convert image to equations
  - image to parametric curves
  - vectomancy CLI usage
  - spline/fourier/chaikin processing
  - video to parametric equations
  - vectomancy video export
---

# Vectomancy CLI

High-performance Rust CLI that parses graphic files and converts them into parametric equations.

## Installation

```bash
# From crates.io
cargo install vectomancy-cli

# From AUR (Arch Linux)
yay -S vectomancy

# From Homebrew (macOS)
brew tap Xuepoo/tap
brew install vectomancy

# From Scoop (Windows)
scoop bucket add Xuepoo https://github.com/Xuepoo/scoop-bucket
scoop install vectomancy
```

## Subcommands

### `image` — Process images

```bash
vectomancy image [OPTIONS] [INPUTS]...
```

**Key flags:**
- `-o, --output <PATH>` — Output file or directory
- `-f, --format <FMT>` — `python`, `html`, `json`, `desmos`, `png`, `jpg`, `webp`
- `-m, --mode <MODE>` — `fourier`, `spline`, `chaikin`
- `--detail <1-100>` — Higher = more equations/slower
- `--terms <N>` — Fourier term count
- `--chaikin-iters <N>` — Chaikin smoothing iterations
- `--min-path-len <N>` — Filter noise (5=clean, 15+=noisy)
- `--color <true|false>` — Enable color sampling
- `--gpu <true|false>` — GPU acceleration

### `text` — Render text as image then process

```bash
vectomancy text [OPTIONS] <TEXT>
```

**Requires `-f, --font <PATH>` for a .ttf/.otf file.**

**Key flags:** `--color <#HEX>`, `--gradient <#A,#B,angle>`, `--letter-spacing <px>`

### `video` — Process video files

```bash
vectomancy video [OPTIONS] <INPUT>
```

**Key flags:** `-o, --output <PATH>`, `-v` for verbose.

**IMPORTANT**: The video subcommand does NOT accept algorithm flags. All settings are read from `~/.config/vectomancy/config.toml` `[image]` section.

**Output modes:**
- Directory output: Generates per-frame HTML/PNG files
- Video output: When output extension is `mp4`, `mkv`, `webm`, `avi`, `mov`, or `gif`, stitches frames into video using ffmpeg

**Video features:**
- Audio preservation: Automatically extracts and merges audio
- Transparent background: Use `.webm` + `bg_transparent = true` in config
- GPU context reuse: Prevents device loss on long videos

## Output Formats

| Format   | Description                                      |
|----------|--------------------------------------------------|
| `json`   | Raw AST with polynomial coefficients per segment |
| `python` | Matplotlib script with compressed equation data  |
| `html`   | Canvas API rendering with per-stroke colors      |
| `desmos` | Desmos Graphing Calculator with LaTeX equations  |
| `png`    | Native rasterized output                         |
| `jpg`    | Rasterized with solid background                 |
| `webp`   | WebP rasterized output                           |

## Quick Examples

```bash
# Spline → JSON (fastest)
vectomancy image photo.jpg -o out.json -f json -m spline --detail 30

# Fourier → Python matplotlib script
vectomancy image logo.png -o out.py -f python -m fourier --terms 20

# Chaikin → HTML canvas
vectomancy image art.svg -o out.html -f html -m chaikin --chaikin-iters 3

# Spline → Desmos calculator
vectomancy image diagram.png -o out.html -f desmos -m spline --detail 15

# With GPU acceleration
vectomancy image large.jpg -o out.json -f json -m spline --gpu true --threads 8

# Process directory of images
vectomancy image ./photos/ -o ./output/ -f json -m spline

# Video → Video (with audio)
vectomancy video input.mp4 -o output.mp4

# Video → WebM (transparent background)
vectomancy video input.mp4 -o output.webm  # set bg_transparent=true in config

# Text → Image
vectomancy text "Hello" --font /path/to/font.ttf -o out.png --color "#FF0000"
```

## Configuration

Config file: `~/.config/vectomancy/config.toml`

```toml
[image]
mode = "spline"           # spline, fourier, chaikin
color = true              # Enable color sampling
gpu = true                # GPU acceleration
threads = 4               # CPU threads
detail = 50               # Detail level (1-100)
min_path_len = 5          # Minimum path length
bg_transparent = false    # Transparent background (for webm)

[video]
enabled = true

[text]
font = "default"
```

## Pitfalls

- `text` subcommand does NOT accept `-m` flag
- Video subcommand reads all settings from config.toml, not CLI flags
- **Do NOT pipe video output** through `grep`, `tail`, etc. — causes SIGPIPE and truncates video
- GPU device loss on long videos is fixed (GPU context reuse)
- For batch processing, use directory input rather than looping single files
- `python` format output can be 2MB+ for complex images
- `desmos` format outputs HTML (not plain LaTeX)
- Build from source: use `cargo build --release` then copy binary, NOT `cargo install`
