# Configurable Smoothing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement configurable `chaikin_iters` to allow users to apply Chaikin smoothing before Fourier or Spline curve generation, configurable via CLI or `config.toml`.

**Architecture:**

1. Port the `chaikin_smooth` function from the `feature/smoothing-chaikin` branch into `src/math/mod.rs`.
2. Add `chaikin_iters` to `cli::RunArgs` and `config::Config`.
3. In `main.rs`, read `chaikin_iters` (falling back to config, then defaulting to 0).
4. Apply `chaikin_smooth` to the extracted paths before passing them to the math functions (e.g. after `simplify_rdp`).

## Phase 1: Core Math & Config/CLI Setup

- [ ] Modify `src/math/mod.rs`: Add the `chaikin_smooth` function (and its test) that was previously developed in the `feature/smoothing-chaikin` branch. You can use git commands to extract it, e.g., `git show feature/smoothing-chaikin:src/math/mod.rs`.
- [ ] Modify `src/cli.rs`: Add `#[arg(short = 'c', long)] pub chaikin_iters: Option<usize>,` to the `RunArgs` struct.
- [ ] Modify `src/config.rs`: Add `pub chaikin_iters: Option<usize>,` to the `Config` struct.

## Phase 2: Integration in Main Logic

- [ ] Modify `src/main.rs`: Load the config using `config::Config::load()`.
- [ ] Modify `src/main.rs`: Resolve the effective `chaikin_iters`: `let iters = args.chaikin_iters.or(config.chaikin_iters).unwrap_or(0);`.
- [ ] Modify `src/main.rs`: Inside the `models::ParserOutput::Paths(paths)` arm, for both `Fourier` and `Spline` modes, apply Chaikin smoothing. Specifically:
  ```rust
  let reduced = math::simplify_rdp(&path, 0.5);
  let smoothed = if iters > 0 {
      math::chaikin_smooth(&reduced, iters)
  } else {
      reduced
  };
  ```
  Use `smoothed` instead of `reduced` for the subsequent `perform_fft` or `spline` steps.
- [ ] Modify `src/main.rs`: Ensure `crate::config` is imported if not already.

## Phase 3: Testing & Verification

- [ ] Build the project to ensure no compilation errors (`cargo build`).
- [ ] Run tests to ensure `chaikin_smooth` is working (`cargo test`).
- [ ] Run the tool with `--mode spline --chaikin-iters 2` on `tests/assets/miku.png` and verify the output script runs successfully without AST explosion and produces a smoothed image.
- [ ] Run the tool with `--mode fourier --chaikin-iters 1` on `tests/assets/teto.png` to verify it works with Fourier mode as well.
- [ ] Check `git diff` and commit the changes.
