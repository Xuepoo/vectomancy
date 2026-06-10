# Typography Design Engine Specification

**Date:** 2026-06-10
**Status:** Proposed
**Context:** The `vectomancy-cli text` subcommand currently parses fonts into splines and can render them to images, but lacks advanced visual styling. This spec outlines the enhancement of the text subcommand into a high-end typography generator.

## 1. Goal
Transform `vectomancy-cli text` into a Typography Design Engine capable of producing stylized, math-art raster images (PNG/WebP) with custom local fonts, solid colors, directional gradients, variable stroke weights, and transparent backgrounds.

## 2. Architecture & Components

### 2.1 CLI Interface (`cli/src/cli.rs`)
Extend `TextArgs` to include:
- `--color <HEX>`: Solid color for the text (e.g., `#FF0000`).
- `--gradient <HEX_START,HEX_END,ANGLE>`: Linear gradient definition. `ANGLE` is in degrees (0 = left to right, 90 = bottom to top).
- `--stroke-width <F32>`: Thickness of the mathematical splines.

### 2.2 AST & Model Updates (`src/models.rs`)
Currently, `ColoredPath` holds `color_rgb: [f32; 3]`. We will introduce a new styling paradigm to support gradients:
```rust
pub enum ColorStyle {
    Solid([f32; 3]),
    LinearGradient {
        start_color: [f32; 3],
        end_color: [f32; 3],
        angle_degrees: f32,
    }
}
```
`ColoredPath` will be updated to use `ColorStyle` instead of a raw float array, or a wrapper will be passed down to the native emitter.

### 2.3 WGPU Rendering Pipeline (`src/emitter/native/`)
- **Shader (`shader.wgsl`)**: The fragment shader must be updated. Instead of a flat `uniform` color, it will calculate its relative position `(x, y)` mapped against the text's global Bounding Box, projecting that onto the gradient's angle vector to interpolate between `start_color` and `end_color`.
- **Bounding Box Calculation**: The emitter must calculate the `(min_x, min_y, max_x, max_y)` of the entire `MathExpressionAST` before pushing vertices to the GPU, passing these limits as uniform variables.

## 3. Data Flow
1. User invokes `vectomancy-cli text "Art" --font ./font.ttf --gradient "#FF0000,#0000FF,45" -o out.png`.
2. Argument parser extracts the string, gradient definition, and stroke weight.
3. `vectomancy_text::parser` reads the TTF and generates geometric segments.
4. Splines are built and packaged into `MathExpressionAST` alongside the parsed `ColorStyle::LinearGradient`.
5. `emitter::native::render_to_image` initializes WGPU:
   - Calculates the global bounding box of the splines.
   - Adds padding to the bounding box equal to `stroke_width / 2.0` to prevent clipping.
   - Uploads gradient colors, angle, and bounding box via Uniform Buffers.
6. The GPU renders the splines; the fragment shader paints the gradient.
7. The output is saved to `out.png`.

## 4. Error Handling & Edge Cases
- **Invalid Colors**: Hex parsing errors (e.g. `#ZZZZZZ` or malformed gradient strings) will halt execution gracefully with a `VectomancyError::InvalidInput`.
- **Clipping**: Mathematical splines can easily exceed canvas bounds when stroke width is large. The canvas dimensions and view matrix MUST be scaled to accommodate `stroke_width`.
- **Format Incompatibility**: If the user asks for a gradient but outputs to JSON or Python, the gradient metadata should be embedded if possible, or gracefully ignored/fallback to a solid color without crashing.

## 5. Testing Strategy
- **Unit Tests**:
  - `cli::parse_gradient`: Verify parsing of `#FF0000,#0000FF,45` into proper f32 arrays and angles.
  - `math::bounding_box`: Verify the AST correctly reports its min/max boundaries.
- **Integration Tests**:
  - Run the CLI with `--gradient` and `--stroke-width 5.0` outputting to `.png`. Verify exit code 0 and file creation.
