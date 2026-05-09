# Project Progress: Vectomancy

## Achievements

- Fixed Ghostty OOM issues by reducing logging verbosity.
- Implemented spline smoothing using a combination of Sobel, Zhang-Suen, RDP, and Spline algorithms.
- Corrected SVG Y-axis inversion.
- Optimized LSP performance by adding # type: ignore and # pylint: disable=all to Python templates.
- Established project infrastructure, including XDG config readiness, Dockerfile, flake.nix, and multi-language READMEs.
- Generated and showcased project assets.

## Roadmap

- Implement TOML config file parsing.
- Develop compute shaders using wgpu or Vulkan for GPU parallelization.
- Create advanced line filtering for colored illustrations.

## Current Challenges

- Complex smoothing algorithms caused AST explosion. The current implementation uses a stable version of Zhang-Suen.
- Large output code files require LSP ignores to maintain editor performance.
