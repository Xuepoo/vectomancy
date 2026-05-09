# Native Formats & LSP Ignores Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate native mathematical software save files (`.ggb` for GeoGebra, `.fkt` for Kmplot) instead of raw text, and add LSP ignore comments to all template files to prevent modern IDEs from freezing when opening generated files.

**Architecture:**

- Add `zip` dependency to create `.ggb` (GeoGebra) archive files dynamically in Rust (`geogebra.xml` inside a zip).
- Create `kmplot.tera` template to output `.fkt` XML format.
- Add Kmplot to the `cli::Format` enum.
- Add ignore directives (e.g., `<!-- htmlmin:ignore -->`, `% chktex-file-disable`, `# pylint: disable=all`) to Tera templates based on the format.

---

## Phase 1: LSP Ignores in Templates

- [ ] Modify `templates/html.tera`
  - Add standard HTML LSP/linter ignore comments at the top (e.g., `<!-- prettier-ignore -->`, `<!-- eslint-disable -->`).
- [ ] Modify `templates/latex.tera`
  - Add TeX ignore directives at the top (e.g., `% !TeX root`, `% chktex-file-disable`, or just `% prettier-ignore`).
- [ ] Modify `templates/python.tera`
  - Ensure `# pylint: disable=all` and `# type: ignore` are at the top to prevent Python language servers from analyzing the generated script.
- [ ] Modify `templates/wolfram.tera`
  - Add standard text/ignore comments if applicable.

## Phase 2: Add Kmplot (.fkt) Support

- [ ] Update `src/cli.rs`
  - Add `Kmplot` to the `Format` enum.
- [ ] Create `templates/kmplot.tera`
  - Write a basic XML template for Kmplot `.fkt` format. The template should iterate over `equations` or `strokes` or `paths` to define `<function>` tags inside the Kmplot XML schema.
- [ ] Update `src/emitter/mod.rs`
  - Register `templates/kmplot.tera` in the Tera engine.
  - Map `Format::Kmplot` to the `kmplot` template.

## Phase 3: Native GeoGebra (.ggb) Support

- [ ] Add dependency
  - Run `cargo add zip` to add the zip archive library.
- [ ] Update `src/emitter/mod.rs`
  - For GeoGebra format, instead of writing the Tera output directly to the output file:
    1. Render `geogebra.tera` (this should now generate valid XML for `geogebra.xml` instead of raw commands).
    2. Create a new ZIP archive at the `output_path`.
    3. Add a file named `geogebra.xml` inside the ZIP archive containing the rendered Tera output.
- [ ] Update `templates/geogebra.tera`
  - Rewrite this template to generate a complete GeoGebra XML structure (`<geogebra format="5.0"> ... <construction> ... <element type="curve"> ...`) instead of raw `Curve(...)` commands.

## Phase 4: Integration Verification

- [ ] Build the project with `cargo build --release`.
- [ ] Run a test generation: `cargo run --release -- run tests/assets/teto.png --output /tmp/agents/vectomancy/test.ggb --format geogebra --mode spline`.
- [ ] Run a test generation: `cargo run --release -- run tests/assets/teto.png --output /tmp/agents/vectomancy/test.fkt --format kmplot --mode spline`.
- [ ] Verify that `/tmp/agents/vectomancy/test.ggb` is a valid ZIP file using the `unzip -l` or `file` command.
- [ ] Verify that the generated `.py`, `.html`, and `.tex` files start with the proper LSP ignore comments.
