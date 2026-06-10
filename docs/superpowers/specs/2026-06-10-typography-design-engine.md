# Typography Design Engine Specification

**Date:** 2026-06-10
**Status:** Proposed
**Context:** The `vectomancy-cli text` subcommand currently parses fonts into splines and can render them to images, but lacks advanced visual styling. This spec outlines the enhancement of the text subcommand into a high-end typography generator.

## 1. Goal
Transform `vectomancy-cli text` into a Typography Design Engine capable of producing stylized, math-art raster images (PNG/WebP) with custom local fonts, solid colors, directional gradients, variable stroke weights, and transparent backgrounds.

## 2. Architecture & Components

### 2.1 CLI Interface (`cli/src/cli.rs` or `src/cli.rs`)
Extend `TextArgs` to include:
- `--color <HEX>`: Solid color for the text (e.g., `#FF0000`).
- `--gradient <HEX_START,HEX_END,ANGLE>`: Linear gradient definition. `ANGLE` is in degrees (0 = left to right, 90 = bottom to top).
- `--stroke-width <F32>`: Thickness of the mathematical splines.

### 2.2 AST & Model Updates (`src/models.rs`)
Currently, `ColoredPath` holds `color_rgb: [f32; 3]`. We will introduce a new styling paradigm to support gradients. The field inside `ColoredPath` will be updated and must use `#[serde(alias = "color_rgb")]` to maintain backward compatibility with old JSON ASTs.
```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)] // For backwards compatibility with old `color_rgb: [f32; 3]` JSON ASTs
pub enum ColorStyle {
    Solid([f32; 3]),
    LinearGradient {
        start: [f32; 3],
        end: [f32; 3],
        angle: f32, // Stored in normalized degrees [0, 360)
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
3. `vectomancy::parser::text` (or equivalent parsing module) reads the TTF and generates geometric segments.
4. Splines are built and packaged into `MathExpressionAST` alongside the parsed `ColorStyle::LinearGradient`.
5. `emitter::native::render_to_image` initializes WGPU:
   - Evaluates `stroke_width`. If `> 0.0`, calculates the polygon expansion on the CPU using `rayon` (or `fold` in WASM), and performs Vertex Batching before uploading to the GPU.
   - Calculates the global bounding box of the splines using `rayon` (or `fold` in WASM).
   - The `angle_degrees` must be normalized to `[0, 360)` using `f32::rem_euclid(360.0)` before uniform upload.
   - Modifies the Orthographic Projection matrix to scale the viewport. The padding must explicitly incorporate the Miter Limit to prevent sharp corners from clipping: `padding = (stroke_width / 2.0) * miter_limit`. DO NOT increase physical Canvas pixel dimensions.
   - Uploads gradient colors, angle, and bounding box via Uniform Buffers.
6. The GPU renders the splines. The engine must use **Lazy Initialization** for its pipelines: instead of always compiling both, it must only compile and bind the specific `wgpu::RenderPipeline` descriptor required for the current run (`LineList` if `stroke_width == 0.0`, or `TriangleList` if `> 0.0`) to avoid unnecessary shader compilation overhead in CLI mode. The fragment shader paints the gradient.
7. The output is saved to `out.png`.

## 4. Error Handling & Edge Cases
- **Invalid Colors**: Hex parsing errors (e.g. `#ZZZZZZ` or malformed gradient strings) will halt execution gracefully with a `VectomancyError::InvalidInput`.
- **Invalid Stroke Width & Hairlines**: Missing or invalid stroke width validation (`< 0.0` or `NaN`) will be rejected. A value of `0.0` will map to WGPU's native `LineList` / `LineStrip` topology (letting the GPU render true 1-pixel hairlines regardless of Viewport scaling), while positive values will be expanded into polygons.
- **Empty AST / Empty String**: If the input string is empty `""`, the parser MUST NOT return an error. It must completely short-circuit the WGPU initialization and memory allocation process, directly returning a `1x1` pixel empty transparent PNG/WebP buffer from memory to avoid WGPU 0-dimension texture panics.
- **Font File Safety**: If the user provides a corrupted, 0-byte, or unsupported font file, the parser must intercept this before panic and return a safe `VectomancyError::FontParseError`.
- **WASM Binding Separation**: To maintain Separation of Concerns, a new dedicated WASM API `process_text(font_bytes: &[u8], text: &str)` MUST be introduced instead of overloading `process_image`.
- **WASM Font Size Safety (Defense-in-depth)**: To prevent FFI memory overflow, a 10MB limit check MUST be implemented in the JavaScript layer. Additionally, the Rust WASM binding must explicitly verify `font_bytes.len() <= 10 * 1024 * 1024` immediately upon entry before any downstream parsing to protect the WebAssembly linear memory heap.
- **WASM OOM Prevention**: The `process_text` WASM binding must enforce a strict hard limit on maximum input characters (e.g., 500) and generated spline nodes to prevent `serde-wasm-bindgen` serialization from exhausting the browser heap.
- **Clipping Safety**: Mathematical splines can easily exceed canvas bounds when stroke width is large. Instead of inflating texture allocations, the View/Projection Matrix will be zoomed out.
- **Format Incompatibility**: If the user asks for a gradient but outputs to JSON or Python, the emitter will fall back to using `start_color` for the solid export color.
- **Animation / Frame Assembly**: Animated WebP generation and out-of-order frame assembly for dynamic typography are strictly Out-of-Scope for this iteration.

## 5. Testing Strategy
- **Unit Tests**:
  - `cli::parse_gradient`: Verify parsing of `#FF0000,#0000FF,45` into proper f32 arrays and angles.
  - `math::bounding_box`: Verify the AST correctly reports its min/max boundaries.
- **Integration Tests**:
  - Run the CLI with `--gradient` and `--stroke-width 5.0` outputting to `.png`. Verify exit code 0 and file creation.
  - **Golden Image Visual Testing:** Use WGPU headless rendering to generate output and compare its pixels/perceptual hash against a known-good Golden Image reference to guarantee Shader gradient interpolations and clipping rules are mathematically precise.
