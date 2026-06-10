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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ColorStyle {
    Solid { color: [f32; 3] },
    LinearGradient {
        start_color: [f32; 3],
        end_color: [f32; 3],
        angle_degrees: f32,
    }
}
```
`ColoredPath` will be updated to use `ColorStyle`.
**Critical:** To prevent WASM serialization breakage, `vectomancy-web/wasm-engine/src/lib.rs` and the frontend `zola-site/templates/app.html` must be updated to handle the new JSON structure, adapting the `ColorStyle` into Canvas API `createLinearGradient` calls.

### 2.3 WGPU Rendering Pipeline (`src/emitter/native/`)
- **Shader (`shader.wgsl`)**: The fragment shader must be updated. Instead of a flat `uniform` color, it will calculate its relative position `(x, y)` mapped against the text's global Bounding Box, projecting that onto the gradient's angle vector to interpolate between `start_color` and `end_color`.
- **Bounding Box Calculation**: The emitter must calculate the `(min_x, min_y, max_x, max_y)` of the entire `MathExpressionAST` before pushing vertices to the GPU. This calculation MUST use `rayon` (`par_iter().reduce()`) for multi-threaded performance on the CPU (using `#[cfg(not(target_arch = "wasm32"))]`), and fallback to a standard single-threaded `.iter().fold()` under `#[cfg(target_arch = "wasm32")]` to ensure WASM compilation succeeds.

## 3. Data Flow
1. User invokes `vectomancy-cli text "Art" --font ./font.ttf --gradient "#FF0000,#0000FF,45" -o out.png`.
2. Argument parser extracts the string, gradient definition, and stroke weight.
3. `vectomancy_text::parser` reads the TTF and generates geometric segments.
4. Splines are built and packaged into `MathExpressionAST` alongside the parsed `ColorStyle::LinearGradient`.
5. `emitter::native::render_to_image` initializes WGPU:
   - Calculates the global bounding box of the splines using `rayon` (or `fold` in WASM).
   - The `angle_degrees` must be normalized to `[0, 360)` modulo 360 before uniform upload.
   - Modifies the Orthographic Projection matrix to scale the viewport to accommodate `stroke_width / 2.0` padding (DO NOT increase physical Canvas pixel dimensions to prevent GPU OOM).
   - Uploads gradient colors, angle, and bounding box via Uniform Buffers.
6. The GPU renders the splines; the fragment shader paints the gradient.
7. The output is saved to `out.png`.

## 4. Error Handling & Edge Cases
- **Invalid Colors**: Hex parsing errors (e.g. `#ZZZZZZ` or malformed gradient strings) will halt execution gracefully with a `VectomancyError::InvalidInput`.
- **Invalid Stroke Width**: Missing or invalid stroke width validation (`< 0.0` or `NaN`) will be rejected at the CLI parsing layer. A value of `0.0` will be explicitly treated as a 1-pixel "hairline" render, while a hard maximum (e.g., `100.0`) will be enforced.
- **Empty AST / Empty String**: If the input string is empty `""`, the parser will immediately return an error or an empty transparent image without passing undefined `NaN` bounding boxes to the GPU.
- **Font File Safety**: If the user provides a corrupted, 0-byte, or unsupported font file, the parser must intercept this before panic and return a safe `VectomancyError::FontParseError`.
- **WASM OOM Prevention**: The `process_image` WASM binding must enforce a strict hard limit on maximum input characters (e.g., 500) and generated spline nodes to prevent `serde-wasm-bindgen` serialization from exhausting the browser heap.
- **Clipping Safety**: Mathematical splines can easily exceed canvas bounds when stroke width is large. Instead of inflating texture allocations, the View/Projection Matrix will be zoomed out.
- **Format Incompatibility**: If the user asks for a gradient but outputs to JSON or Python, the emitter will fall back to using `start_color` for the solid export color.

## 5. Testing Strategy
- **Unit Tests**:
  - `cli::parse_gradient`: Verify parsing of `#FF0000,#0000FF,45` into proper f32 arrays and angles.
  - `math::bounding_box`: Verify the AST correctly reports its min/max boundaries.
- **Integration Tests**:
  - Run the CLI with `--gradient` and `--stroke-width 5.0` outputting to `.png`. Verify exit code 0 and file creation.
  - **Golden Image Visual Testing:** Use WGPU headless rendering to generate output and compare its pixels/perceptual hash against a known-good Golden Image reference to guarantee Shader gradient interpolations and clipping rules are mathematically precise.
