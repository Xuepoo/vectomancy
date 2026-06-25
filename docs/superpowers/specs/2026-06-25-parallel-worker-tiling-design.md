# Design Spec: Parallel Web Worker Image Tiling for Vectomancy

**Date:** 2026-06-25
**Author:** Antigravity (Advanced Agentic Coding Agent)
**Status:** Draft / Pending Review
**Branch:** `feat/vecto-ui-rebuild`
**Target Repositories:** `vectomancy-web` (Zola site & WASM-engine)

---

## 1. Executive Summary

This spec outlines the architecture for introducing multi-core parallel processing to image vectorization in the browser. High-resolution raster images (e.g., 5000x5000) will be split into non-overlapping tile segments, processed concurrently in separate Web Workers running the compiled Rust WASM engine, and merged back into a unified mathematical AST on the main thread for rendering.

---

## 2. API & Data Communication Contracts

### 2.1 Main Thread to Worker Messages

Communication between the main thread and the workers uses standard `postMessage` with structured messaging.

#### `init` Message
Sent once upon worker creation to initialize the WASM runtime:
```typescript
interface InitMessage {
  type: 'init';
  wasmUrl: string; // Absolute URL to the `.wasm` file
  jsWrapperUrl: string; // Absolute URL to the `.js` glue wrapper
}
```

#### `process` Message
Sent to trigger vectorization of a single tile:
```typescript
interface ProcessMessage {
  type: 'process';
  tileId: number;
  pixelBuffer: ArrayBuffer; // Transferred RGBA buffer
  width: number;
  height: number;
  offsetX: number;
  offsetY: number;
  config: {
    mode: 'spline' | 'fourier' | 'chaikin';
    color: boolean;
    colorDepth: number;
    simplifyMath: boolean;
    detailThreshold: number;
    minPathLength: number;
    fourierAdaptive: boolean;
    fourierEnergyThreshold: number;
    // ...other parser configurations
  };
}
```

### 2.2 Worker to Main Thread Messages

#### `success` Message
Returned upon successful processing of a tile:
```typescript
interface SuccessMessage {
  type: 'success';
  tileId: number;
  ast: {
    paths: Array<{
      color: string;
      commands: Array<{
        type: 'M' | 'L' | 'C' | 'Z';
        x?: number;
        y?: number;
        x1?: number;
        y1?: number;
        x2?: number;
        y2?: number;
      }>;
    }>;
  };
}
```

#### `error` Message
Returned if WASM initialization or processing fails:
```typescript
interface ErrorMessage {
  type: 'error';
  tileId: number;
  error: string;
}
```

---

## 3. Worker Lifecycle & WASM Loading

To ensure cross-browser compatibility and prevent CORS/path resolution issues:
1. **Instantiation**: Workers are created dynamically from a dedicated script `/js/image-worker.js` as module workers:
   ```javascript
   const worker = new Worker('/js/image-worker.js', { type: 'module' });
   ```
2. **WASM Initialization**:
   Inside the worker, the WASM wrapper is loaded via standard ES imports. Upon receiving the `init` message, the worker calls the wrapper's `init()` function passing the absolute `wasmUrl`:
   ```javascript
   import init, { process_image } from '/wasm/wasm_engine.js';

   self.onmessage = async (e) => {
     if (e.data.type === 'init') {
       await init(e.data.wasmUrl);
       self.postMessage({ type: 'initialized' });
     }
     // ...
   };
   ```

---

## 4. Tiling & Stitching Algorithm

### 4.1 Image Partitioning (Main Thread)
For an image with dimensions $W \times H$:
1. Determine tile grid dimensions $M \times N$ (defaulting to 4 tiles for $2 \times 2$ grid on large images, scaling dynamically up to the host system's hardware concurrency limit via `navigator.hardwareConcurrency`).
2. Calculate individual tile size:
   $$W_{\text{tile}} = \lceil W / M \rceil, \quad H_{\text{tile}} = \lceil H / N \rceil$$
3. Crop the raw input buffer to retrieve pixel arrays for each tile.
4. Dispatch each tile as a `ProcessMessage` transferring its raw pixel `ArrayBuffer`.

### 4.2 AST Merging & Offset Correction
When a worker returns a `SuccessMessage`, the coordinates inside its `ast.paths` are local to the tile bounds ($0$ to $W_{\text{tile}}$, $0$ to $H_{\text{tile}}$). The main thread shifts these points back to the global coordinates:
```typescript
function mergeTileAST(tileAst: TileAST, offsetX: number, offsetY: number) {
  for (const path of tileAst.paths) {
    for (const cmd of path.commands) {
      if (cmd.x !== undefined) cmd.x += offsetX;
      if (cmd.y !== undefined) cmd.y += offsetY;
      if (cmd.x1 !== undefined) cmd.x1 += offsetX;
      if (cmd.y1 !== undefined) cmd.y1 += offsetY;
      if (cmd.x2 !== undefined) cmd.x2 += offsetX;
      if (cmd.y2 !== undefined) cmd.y2 += offsetY;
    }
  }
}
```

---

## 5. Error Handling & Fallbacks

- **WASM Loading Fallback**: If module workers are not supported by the browser (e.g. older Safari), the application falls back automatically to executing `process_image` sequentially on the main thread.
- **Worker Crashes & OOM**: If a worker crashes or encounters an Out-Of-Memory (OOM) error (detected via `error` events), the dispatcher aborts the current tiling task, notifies the user via an error dialog, and falls back to main-thread processing.
- **Processing Timeouts**: If a worker takes longer than 15 seconds to return a result, the main thread terminates the worker, spawns a replacement, and raises a timeout warning.

---

## 6. Verification and Testing Plan

1. **Stitching Precision Test**: Unit test verifying that points in local coordinates $[0, 0]$ on tile $(2500, 2500)$ are correctly shifted to $[2500, 2500]$ globally.
2. **Worker Concurrency E2E Test**:
   - Mock WASM processing inside workers with mock timers.
   - Assert that multiple worker tasks are dispatched simultaneously and aggregated in order.
