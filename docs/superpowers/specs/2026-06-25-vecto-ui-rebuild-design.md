# Design Spec: Full Canvas UI Rebuild & Parallel Web Worker Tiling

**Date:** 2026-06-25
**Author:** Antigravity (Advanced Agentic Coding Agent)
**Status:** Draft / Pending Review
**Branch:** `feat/vecto-ui-rebuild`
**Target Repositories:** `vectomancy-web` & `vecto-ui`

---

## 1. Executive Summary

This document specifies the architectural redesign and implementation details for:
1. **Full Canvas UI Rebuild**: Migrating the `vectomancy-web` frontend layout and setting controls (sliders, select dropdowns, inputs, buttons, containers) from standard HTML + `nes.css` DOM layout to a single full-screen `<canvas>` driven by the `vecto-ui` Canvas ECS framework.
2. **Parallel Web Worker Tiling**: Partitioning large image inputs (up to 5000x5000) into smaller tiles, distributing the mathematical processing (WASM fitters) to parallel Web Workers, and aggregating the resulting Vector ASTs back in the main thread for high-precision rendering.

---

## 2. System Architecture

```mermaid
graph TD
    User([User Screen]) -->|Pointer Events| DOMShadow[DOM Shadow Layer: a11yRoot]
    DOMShadow -->|Captured Focus/Input| CanvasECS[vecto-ui Canvas ECS Scene]
    
    subgraph UI Components (vecto-ui)
        CanvasECS --> Sidebar[NESContainerEntity: Sidebar]
        CanvasECS --> Viewport[NESContainerEntity: Viewport]
        
        Sidebar --> Slider[NESSliderEntity]
        Sidebar --> Checkbox[NESCheckboxEntity]
        Sidebar --> Select[NESSelectEntity]
        Sidebar --> Button[NESButtonEntity]
        
        Viewport --> Topbar[NESTopBarEntity]
        Viewport --> VectorView[MathVectorViewEntity]
        Viewport --> Gallery[NESGalleryEntity]
    end

    subgraph Parallel Fitter (Web Workers)
        VectorView -->|Split Tiles| Dispatcher[Worker Dispatcher]
        Dispatcher -->|Tile 1| Worker1[Web Worker 1: WASM Engine]
        Dispatcher -->|Tile 2| Worker2[Web Worker 2: WASM Engine]
        Dispatcher -->|Tile N| WorkerN[Web Worker N: WASM Engine]
        
        Worker1 -->|AST Tile 1| Dispatcher
        Worker2 -->|AST Tile 2| Dispatcher
        WorkerN -->|AST Tile N| Dispatcher
        Dispatcher -->|Merge & Offset Shift| VectorView
    end
```

---

## 3. Hybrid DOM Interaction Layer (`a11yRoot`)

To maintain full input method editor (CJK IME) compatibility, mobile keyboard popup support, copy-paste capabilities, and accessibility, `vecto-ui` uses a transparent DOM Shadow Layer.

- **Focus & Composition Capture**: Every interactive Canvas UI entity (e.g. `NESInputEntity`) has a corresponding invisible HTML input element (`<input type="text">` styled with `opacity: 0; pointer-events: auto; z-index: 10;`) positioned precisely over the Canvas bounding box.
- **Event Forwarding**:
  - Focus is handled natively by the browser when clicking the element.
  - Text composition (IME) and characters are captured by the transparent input and synced to the Canvas entity on `input` events.
  - Caret/Cursor positioning and text selections are computed and drawn on Canvas while matching the DOM text selection state.

---

## 4. Parallel Web Worker Tiling (Image Partitioning)

### 4.1 Algorithm Description

1. **Partitioning**: Given an image of width $W$, height $H$, and tile counts $M \times N$:
   - Calculate tile dimensions: $W_{\text{tile}} = \lfloor W / M \rfloor$, $H_{\text{tile}} = \lfloor H / N \rfloor$.
   - Extract image pixel sub-arrays (rgba) for each tile.
2. **Distribution**: Spawn $K = M \times N$ Web Workers. Send each worker its pixel buffer, dimensions, and the coordinate offsets $(X_{\text{offset}}, Y_{\text{offset}})$.
3. **Execution**: Workers initialize the WASM engine and run `process_image(pixel_buffer)`.
4. **Aggregation**:
   - Workers return the generated Vector AST (composed of Bezier splines or Fourier term paths).
   - The main thread receives the ASTs.
   - For each path in a tile's AST, shift the points:
     $$x_{\text{global}} = x_{\text{local}} + X_{\text{offset}}$$
     $$y_{\text{global}} = y_{\text{local}} + Y_{\text{offset}}$$
   - Merge all paths into a single unified `PathAST` representation and register it into the `MathVectorViewEntity`.

### 4.2 Algorithmic Complexity

- **Time Complexity**:
  - Image Splitting: $O(W \times H)$ to slice the image buffers.
  - Worker Fitting: If the total number of pixels is $N = W \times H$, a single-threaded TSP pathing takes $O(N \log N)$. Running $K$ workers on tiles of size $N/K$ reduces wall-clock execution time to:
    $$O\left(\frac{N}{K} \log \frac{N}{K}\right)$$
    This yields a near-linear speedup on systems with $\ge K$ physical CPU cores.
  - AST Stitching: $O(P)$ where $P$ is the total number of generated curve paths.
- **Space Complexity**:
  - $O(W \times H)$ to hold tiled pixel buffers. Use of **Transferables** allows zero-copy passing of `ArrayBuffers` between threads, preventing garbage collection overhead.

---

## 5. UI Component Implementations (`@vecto/ui`)

The UI components will emulate the classic 8-bit retro aesthetic (`nes.css`) using Canvas 2D drawing primitives:

- **Border Rendering**: 2px thick black outline drawn with double borders.
- **Font Rendering**: Custom `Press Start 2P` font rendered with pixelated scaling.
- **Button Entity**: Computes hover offsets and active states. Redraws only on pointer events.
- **Slider Entity**: Renders track and square handle. Syncs value changes to the linked configuration parameters.
- **Dropdown (Select) Entity**: Draws select box. On click, focuses a transparent HTML `<select>` tag in the shadow layer to open native option picker.

---

## 6. Verification and Testing Plan

1. **Unit Tests**:
   - Write tests for tile coordinates translation: Verify that tiles are stitched back with correct absolute positions.
   - Test worker dispatcher: Mock Web Worker communication and verify AST merging.
2. **E2E Browser Validation**:
   - Run browser tests using Headless Chrome.
   - Select text inputs and sliders via CSS selectors in `a11yRoot` and check canvas redraw correctness.
   - Audit visual layout against the original `nes.css` design parameters.
