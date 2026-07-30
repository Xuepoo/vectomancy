# Vectomancy Workspace Decoupling & Multi-Crate Architecture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decouple `vectomancy` into modular, domain-focused Rust workspace crates (`vectomancy-geometry`, `vectomancy-transform`, `vectomancy-raster`, `vectomancy-svg`, `vectomancy-export`, `vectomancy-pipeline`, `facade/vectomancy`), removing monolithic coupling while preserving full SemVer backward compatibility.

**Architecture:** Split the monolithic `vectomancy` root crate into layered workspace crates under `crates/` and `facade/`. `vectomancy-geometry` serves as the lightweight, zero-heavy-dependency core containing foundational primitive types (`Point2D`, `Polyline`, `BoundingBox`, `StyledPath`, `Scene`) and pure geometric algorithms (`RDP`, `Chaikin`, `Arc-length Resampling`). Higher-level crates build upon `geometry`, and `facade/vectomancy` re-exports everything with `#[deprecated]` notices for legacy paths to ensure zero breaking changes for existing consumers.

**Tech Stack:** Rust 2021, Cargo Workspaces, Rayon, Serde, Lyon, CarryCtx.

---

### Task 1: Create `vectomancy-geometry` Crate & Primitive Types (CTX-0001)

**Files:**
- Create: `crates/vectomancy-geometry/Cargo.toml`
- Create: `crates/vectomancy-geometry/src/lib.rs`
- Create: `crates/vectomancy-geometry/src/types.rs`
- Create: `crates/vectomancy-geometry/src/algorithms/rdp.rs`
- Create: `crates/vectomancy-geometry/src/algorithms/chaikin.rs`
- Create: `crates/vectomancy-geometry/src/algorithms/resampling.rs`
- Create: `crates/vectomancy-geometry/tests/geometry_tests.rs`
- Modify: `Cargo.toml` (workspace members)

- [ ] **Step 1: Write failing integration tests for `vectomancy-geometry`**

Create `crates/vectomancy-geometry/tests/geometry_tests.rs`:
```rust
use vectomancy_geometry::{
    chaikin_smooth, resample_by_arc_length, simplify_rdp, BoundingBox, Point2D, Polyline,
};

#[test]
fn test_rdp_simplification() {
    let points = vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 1.0, y: 0.1 },
        Point2D { x: 2.0, y: 0.0 },
    ];
    let simplified = simplify_rdp(&points, 0.2);
    assert_eq!(simplified.len(), 2);
    assert_eq!(simplified[0], Point2D { x: 0.0, y: 0.0 });
    assert_eq!(simplified[1], Point2D { x: 2.0, y: 0.0 });
}

#[test]
fn test_chaikin_smooth() {
    let points = vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 4.0, y: 0.0 },
        Point2D { x: 4.0, y: 4.0 },
    ];
    let polyline = Polyline {
        points,
        closed: false,
    };
    let smoothed = chaikin_smooth(&polyline, 1);
    assert!(smoothed.points.len() > 3);
}

#[test]
fn test_arc_length_resampling() {
    let polyline = Polyline {
        points: vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 10.0, y: 0.0 },
        ],
        closed: false,
    };
    let resampled = resample_by_arc_length(&polyline, 1.0);
    assert_eq!(resampled.points.len(), 11);
}
```

- [ ] **Step 2: Create `crates/vectomancy-geometry/Cargo.toml` and implement data types & algorithms**

Write `crates/vectomancy-geometry/src/lib.rs` and submodules implementing `Point2D`, `BoundingBox`, `Polyline`, `StyledPath`, `Scene`, `simplify_rdp`, `chaikin_smooth`, and `resample_by_arc_length`.

- [ ] **Step 3: Run `cargo test -p vectomancy-geometry` to verify passing tests**

Run: `cargo test -p vectomancy-geometry`
Expected: `test test_rdp_simplification ... ok`, `test test_chaikin_smooth ... ok`, `test test_arc_length_resampling ... ok`.

- [ ] **Step 4: Commit**

```bash
git add crates/vectomancy-geometry Cargo.toml
git commit -m "feat(geometry): extract standalone vectomancy-geometry crate (CTX-0001)"
```

---

### Task 2: Create `vectomancy-transform` Crate (CTX-0002)

**Files:**
- Create: `crates/vectomancy-transform/Cargo.toml`
- Create: `crates/vectomancy-transform/src/lib.rs`
- Create: `crates/vectomancy-transform/src/spline.rs`
- Create: `crates/vectomancy-transform/src/fourier.rs`
- Create: `crates/vectomancy-transform/src/tsp.rs`
- Create: `crates/vectomancy-transform/src/gpu.rs`
- Create: `crates/vectomancy-transform/tests/transform_tests.rs`

- [ ] **Step 1: Write unit tests for `vectomancy-transform`**
- [ ] **Step 2: Migrate Spline, Fourier, TSP, and GPU math into `crates/vectomancy-transform`**
- [ ] **Step 3: Run `cargo test -p vectomancy-transform`**
- [ ] **Step 4: Commit**

```bash
git add crates/vectomancy-transform
git commit -m "feat(transform): extract standalone vectomancy-transform crate (CTX-0002)"
```

---

### Task 3: Create `vectomancy-raster` and `vectomancy-svg` Crates (CTX-0003)

**Files:**
- Create: `crates/vectomancy-raster/Cargo.toml`
- Create: `crates/vectomancy-raster/src/lib.rs`
- Create: `crates/vectomancy-svg/Cargo.toml`
- Create: `crates/vectomancy-svg/src/lib.rs`

- [ ] **Step 1: Write input decoder tests for raster and SVG**
- [ ] **Step 2: Move Image skeletonization (Sobel, Otsu, Zhang-Suen) and SVG path parsing to `crates/vectomancy-raster` & `crates/vectomancy-svg`**
- [ ] **Step 3: Run `cargo test -p vectomancy-raster` and `cargo test -p vectomancy-svg`**
- [ ] **Step 4: Commit**

```bash
git add crates/vectomancy-raster crates/vectomancy-svg
git commit -m "feat(input): extract vectomancy-raster and vectomancy-svg crates (CTX-0003)"
```

---

### Task 4: Create `vectomancy-export` Crate (CTX-0004)

**Files:**
- Create: `crates/vectomancy-export/Cargo.toml`
- Create: `crates/vectomancy-export/src/lib.rs`
- Create: `crates/vectomancy-export/src/json.rs`
- Create: `crates/vectomancy-export/src/svg.rs`
- Create: `crates/vectomancy-export/src/native_image.rs`

- [ ] **Step 1: Write encoder tests**
- [ ] **Step 2: Extract JSON, SVG, Native Image encoders into `crates/vectomancy-export`**
- [ ] **Step 3: Run `cargo test -p vectomancy-export`**
- [ ] **Step 4: Commit**

```bash
git add crates/vectomancy-export
git commit -m "feat(export): extract standalone vectomancy-export crate (CTX-0004)"
```

---

### Task 5: Create `vectomancy-pipeline` Crate (CTX-0005)

**Files:**
- Create: `crates/vectomancy-pipeline/Cargo.toml`
- Create: `crates/vectomancy-pipeline/src/lib.rs`

- [ ] **Step 1: Write end-to-end pipeline conversion tests**
- [ ] **Step 2: Extract conversion pipeline orchestration into `crates/vectomancy-pipeline`**
- [ ] **Step 3: Run `cargo test -p vectomancy-pipeline`**
- [ ] **Step 4: Commit**

```bash
git add crates/vectomancy-pipeline
git commit -m "feat(pipeline): extract standalone vectomancy-pipeline crate (CTX-0005)"
```

---

### Task 6: Refactor Facade Crate & Update CLI / Text / Video (CTX-0006)

**Files:**
- Modify: `src/lib.rs` (Root facade crate re-exports and deprecation warnings)
- Modify: `cli/Cargo.toml` & `cli/src/main.rs`
- Modify: `text/Cargo.toml` & `text/src/lib.rs`
- Modify: `video/Cargo.toml` & `video/src/lib.rs`

- [ ] **Step 1: Update root `vectomancy` crate to serve as facade re-exporting subcrates**
- [ ] **Step 2: Update `vectomancy-cli`, `vectomancy-text`, and `vectomancy-video` to use modular workspace dependencies**
- [ ] **Step 3: Run `cargo test --workspace` to verify full workspace compilation and test suite passing**
- [ ] **Step 4: Commit**

```bash
git add .
git commit -m "refactor(facade): complete workspace decoupling and update downstream crates (CTX-0006)"
```
