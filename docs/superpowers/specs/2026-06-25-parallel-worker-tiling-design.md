# Design Spec: Path-Parallel Web Worker Fitting for Vectomancy

**Date:** 2026-06-25
**Author:** Antigravity (Advanced Agentic Coding Agent)
**Status:** Draft / Pending Review
**Branch:** `feat/vecto-ui-rebuild`
**Target Repositories:** `vectomancy-web` (Zola site & WASM-engine), `vectomancy` (core Rust library)

---

## 1. Executive Summary

This spec outlines the architecture for introducing multi-core parallel processing to image vectorization in the browser.

Instead of dividing the raw raster image into tiles (which introduces complex coordinate offsets, boundary tangent discontinuities in Splines, and Gibbs-phenomenon ripples in Fourier fitting), we adopt a **Path-Parallel** approach:
1. **Contour Extraction (Main Thread)**: The main thread runs the initial contour tracing/path extraction on the image, yielding a list of simplified polylines/paths. This step is extremely fast (typically <10ms for standard images).
2. **Chunk Distribution (Web Workers)**: The extracted paths are split into $K$ balanced chunks (load-balanced by total point count). Each chunk is sent to a Web Worker running the Rust WASM engine.
3. **Off-Thread Fitting (Web Workers)**: The Workers perform the computationally expensive mathematical fitting (FFT for Fourier, cubic Bezier curve fitting for Splines) in parallel.
4. **AST Concatenation (Main Thread)**: The main thread merges the resulting sub-ASTs into a single global AST for immediate rendering.

---

## 2. API & Data Communication Contracts

### 2.1 Rust WASM API Extensions

We will expose a new WASM-bindgen interface in `wasm-engine/src/lib.rs` specifically for fitting pre-extracted paths:

```rust
#[wasm_bindgen]
pub fn fit_paths(paths_json: JsValue, options: JsValue) -> Result<JsValue, JsValue>;
```

#### Struct Serializations (`models/mod.rs`)
To support deserializing paths inside the WASM engine, `Point2D` and `ColoredPath<T>` must derive `serde::Deserialize`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColoredPath<T> {
    #[serde(rename = "color_rgb")]
    pub color_style: Option<ColorStyle>,
    pub data: T,
}
```

#### Dedicated Options Struct (`wasm-engine/src/lib.rs`)
To prevent deserialization failures due to missing image-specific fields like `format` and `color` (which are mandatory in `ProcessOptions`), a dedicated `FitOptions` struct will be introduced:

```rust
#[derive(Deserialize)]
pub struct FitOptions {
    pub mode: String,
    pub chaikin_iters: usize,
    pub terms: usize,
    pub detail: usize,
    pub min_path_len: usize,
    #[serde(default)]
    pub simplify_math: Option<bool>,
    #[serde(default)]
    pub fourier_adaptive: Option<bool>,
    #[serde(default)]
    pub fourier_energy_threshold: Option<f64>,
}
```

---

### 2.2 Web Worker Message Interfaces

#### Color Style definition
```typescript
type ColorStyle =
  | [number, number, number] // Solid RGB color
  | {
      stops: Array<[number, [number, number, number]]>; // Gradient stops: [position, [r, g, b]]
      start_pos: [number, number]; // [x, y] in range [0, 1]
      end_pos: [number, number];   // [x, y] in range [0, 1]
    };
```

#### Main Thread to Worker: `fit` Message
```typescript
interface FitMessage {
  type: 'fit';
  chunkId: number;
  paths: Array<{
    color_rgb: ColorStyle | null;
    data: Array<{ x: number; y: number }>;
  }>;
  config: {
    mode: 'spline' | 'fourier' | 'chaikin';
    chaikin_iters: number;
    terms: number;
    detail: number;
    min_path_len: number;
    simplify_math?: boolean;
    fourier_adaptive?: boolean;
    fourier_energy_threshold?: number;
  };
}
```

#### Worker to Main Thread: `success` Message
Returns a partial `MathExpressionAST` matching the requested mode:
```typescript
interface SuccessMessage {
  type: 'success';
  chunkId: number;
  astChunk: {
    type: 'Fourier' | 'Spline' | 'Polyline';
    strokes?: Array<ColoredPath<Array<FourierTerm>>>;
    equations?: Array<ColoredPath<Array<SplineEquation>>>;
    paths?: Array<ColoredPath<Array<Point2D>>>;
  };
}
```

#### Worker to Main Thread: `error` Message
```typescript
interface ErrorMessage {
  type: 'error';
  chunkId: number;
  error: string;
}
```

---

## 3. Worker Lifecycle & WASM Loading

To comply with CSP restrictions and browser sandbox limits:
1. **Dynamic Import**: Workers are instantiated as ESM module workers:
   ```javascript
   const worker = new Worker('/js/image-worker.js', { type: 'module' });
   ```
2. **Initialization**: On creation, the main thread sends an `init` message with `wasmUrl`. The worker imports the WASM ES binder statically and triggers instantiation:
   ```javascript
   import init, { fit_paths } from '/wasm/wasm_engine.js';

   let isInitialized = false;

   self.onmessage = async (e) => {
     if (e.data.type === 'init') {
       try {
         await init(e.data.wasmUrl);
         isInitialized = true;
         self.postMessage({ type: 'initialized' });
       } catch (err) {
         self.postMessage({ type: 'error', error: `WASM Init Failed: ${err.message}` });
       }
     } else if (e.data.type === 'fit') {
       if (!isInitialized) {
         self.postMessage({ type: 'error', chunkId: e.data.chunkId, error: 'Worker not initialized' });
         return;
       }
       try {
         const result = fit_paths(e.data.paths, e.data.config);
         self.postMessage({ type: 'success', chunkId: e.data.chunkId, astChunk: result.ast });
       } catch (err) {
         self.postMessage({ type: 'error', chunkId: e.data.chunkId, error: `Fitting Failed: ${err}` });
       }
     }
   };
   ```

---

## 4. Work Distribution & Stitching

### 4.1 Path Partitioning & Load Balancing
To maximize multi-core utilization:
1. The main thread runs `process_image` in `polyline` (raw path) mode to extract all path contours.
2. The total number of points across all paths is calculated: $P_{\text{total}} = \sum \text{path.data.length}$.
3. We target $K = \min(\text{navigator.hardwareConcurrency}, 4)$ workers.
4. We distribute the paths into $K$ chunks such that the sum of points in each chunk is roughly equal (greedy bin-packing approach) to prevent worker idle time due to stragglers.

### 4.2 AST Reassembly
Since the paths were extracted globally, no coordinate shifts are required. The global `bounding_box` is already computed by the initial main thread polyline path extraction step, which is passed directly to the combiner.

```typescript
function combineASTChunks(
  chunks: Array<SuccessMessage['astChunk']>,
  globalBoundingBox: [number, number, number, number]
): MathExpressionAST {
  const type = chunks[0].type;

  if (type === 'Fourier') {
    const strokes = chunks.flatMap(c => c.strokes || []);
    return { type: 'Fourier', strokes, bounding_box: globalBoundingBox };
  } else if (type === 'Spline') {
    const equations = chunks.flatMap(c => c.equations || []);
    return { type: 'Spline', equations, bounding_box: globalBoundingBox };
  } else {
    const paths = chunks.flatMap(c => c.paths || []);
    return { type: 'Polyline', paths, bounding_box: globalBoundingBox };
  }
}
```

---

## 5. Error Handling & Fallbacks

- **ESM Worker Fallback**: For browsers that do not support ESM Web Workers, vectorization falls back gracefully to synchronous execution on the main thread.
- **Worker Crash/OOM**: If a worker terminates unexpectedly (e.g. OOM on extremely complex curves), the dispatcher aborts the parallel pipeline and falls back to main-thread rendering.
- **Timeout Protection**: If a worker does not respond within 15 seconds, it is terminated, and a new one is spawned to replace it.

---

## 6. Verification and Testing Plan

1. **Rust Unit Tests**: Verify that `fit_paths` correctly parses input JSON paths and outputs correct Fourier/Spline terms matching the sequential pipeline.
2. **JS Chunking Test**: Unit test verifying the greedy bin-packing path distributor correctly groups paths of varying lengths into balanced chunks.
3. **E2E AST Reassembly Test**: Verify that the assembled global AST is identical to the AST produced by a single-threaded run.
