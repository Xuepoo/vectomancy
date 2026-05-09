# Curve Smoothing Experiment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use opencode:subagent-driven-development (recommended) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement two curve smoothing approaches (Chaikin and Catmull-Rom Bezier) on separate git branches for evaluation against the "staircase/aliasing" effect.

**Architecture:**

- The Chaikin approach will be an iterative corner-cutting pass on raw point arrays in `vectomancy/src/math/mod.rs`, retaining the existing `LineTo` emitter logic.
- The Bezier approach will compute Catmull-Rom spline control points in `vectomancy/src/math/spline.rs` to generate mathematically smooth `CubicTo` segments, requiring emitter logic to handle these segments.

**Note on Git:** We are using standard Git branches (`feature/smoothing-chaikin` and `feature/smoothing-bezier`) within the main workspace, not git worktrees.

---

## Phase 1: Setup & Chaikin Smoothing Branch

- [ ] Run `git checkout -b feature/smoothing-chaikin main` to start the first branch.
- [ ] Open `vectomancy/src/math/mod.rs` and read the current `simplify_rdp` implementation.
- [ ] In `vectomancy/src/math/mod.rs`, implement `pub fn chaikin_smooth(points: &[Point2D], iterations: usize) -> Vec<Point2D>`.
  - Handle edge cases: if `points.len() < 3`, return the original points.
  - Implement the standard Chaikin corner cutting algorithm (cutting at 25% and 75% of each segment).
- [ ] Write a failing test in `vectomancy/src/math/mod.rs` or `vectomancy/tests/math_tests.rs` for `chaikin_smooth`.
- [ ] Ensure `cargo test` fails.
- [ ] Fix `chaikin_smooth` so the test passes.
- [ ] Open `vectomancy/src/parser/raster.rs` and locate where `simplify_rdp` is called (or where paths are finalized).
- [ ] Update `vectomancy/src/parser/raster.rs` to call `chaikin_smooth(..., 2)` on the paths before returning them.
- [ ] Run `cargo build` and `cargo test` to ensure integration works.
- [ ] Run the CLI tool against `vectomancy/tests/assets/miku.png` and output to `/tmp/agents/vectomancy/miku_chaikin.py`.
- [ ] Commit the changes: `git add . && git commit -m "feat: implement chaikin curve smoothing"`.

---

## Phase 2: Catmull-Rom Bezier Branch

- [ ] Run `git checkout main` to return to the baseline.
- [ ] Run `git checkout -b feature/smoothing-bezier main` to start the second branch.
- [ ] Open `vectomancy/src/math/spline.rs` and read the `build_splines` and related segment structures.
- [ ] Create a new function `pub fn fit_cubic_bezier(points: &[Point2D]) -> Vec<crate::models::BezierSegment>` in `vectomancy/src/math/spline.rs`.
  - Handle edge cases: if `points.len() < 3`, fallback to `LineTo`.
  - Calculate Catmull-Rom control points to output smooth `CubicTo` segments.
- [ ] Write a test in `vectomancy/src/math/spline.rs` or `math_tests.rs` for `fit_cubic_bezier`.
- [ ] Run `cargo test` to ensure it fails/passes as implemented.
- [ ] Open `vectomancy/src/parser/raster.rs` and update path generation to use `fit_cubic_bezier` to emit proper bezier segments instead of discrete lines, if necessary (or update the orchestration logic in `src/main.rs` depending on where `BezierSegment` conversion happens).
- [ ] Run `cargo build` and `cargo test`.
- [ ] Run the CLI tool against `vectomancy/tests/assets/miku.png` and output to `/tmp/agents/vectomancy/miku_bezier.py`.
- [ ] Commit the changes: `git add . && git commit -m "feat: implement catmull-rom bezier fitting"`.

---

## Phase 3: Evaluation

- [ ] Review `/tmp/agents/vectomancy/miku_chaikin.py` and `/tmp/agents/vectomancy/miku_bezier.py`.
- [ ] Compare file sizes and visual outputs (e.g. running the python scripts to see the plot).
- [ ] Provide a summary report to the user to choose the best approach or how to integrate them both as CLI options.
