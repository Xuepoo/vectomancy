# AST Explosion Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Modify the Rust emitter to output math parameters as a single Zlib-compressed, Base64-encoded string to eliminate IDE AST explosion, and update Python templates to decode this string at runtime.

**Architecture:** The `emitter` module in `vectomancy/src/` will use `serde_json`, `flate2`, and `base64` to compress mathematical structures into a string. The `vectomancy/templates/python.tera` file will be modified to include decoding logic (`json`, `zlib`, `base64`) with robust error handling.

## File Structure

**Modified Files:**

- `vectomancy/Cargo.toml`: Add dependencies (`flate2`, `base64`, `serde_json`).
- `vectomancy/src/emitter/mod.rs` (or equivalent file handling the Tera context): Add compression logic and pass `encoded_data`.
- `vectomancy/templates/python.tera`: Update rendering script.

**New Files:**

- None (modifying existing files).

## Tasks

### Phase 1: Setup and Dependencies

- [ ] Add `flate2` and `base64` to `vectomancy/Cargo.toml` dependencies.
- [ ] Ensure `serde_json` is present in `Cargo.toml`.
- [ ] Run `cargo fetch` to verify dependencies.
- [ ] Commit: "build: Add flate2 and base64 dependencies for output compression"

### Phase 2: Rust Emitter Compression Logic

- [ ] Open `vectomancy/src/emitter/mod.rs` (or the specific file where Tera context is built, e.g. `vectomancy/src/emitter/context.rs`).
- [ ] Implement a helper function `encode_math_data<T: Serialize>(data: &T) -> Result<String, VectomancyError>` that:
  - Serializes `data` to JSON using `serde_json::to_string`.
  - Compresses the JSON string bytes using `flate2::write::ZlibEncoder`.
  - Encodes the compressed bytes using `base64::engine::general_purpose::STANDARD.encode`.
- [ ] Write a unit test `test_encode_math_data` in the same file to verify the encoding output is correct and stable.
- [ ] Run `cargo test` to verify the new helper function.
- [ ] Update the `Tera` context generation to pass `encoded_data` instead of the raw structures. Ensure error propagation is handled cleanly.
- [ ] Commit: "feat(emitter): Implement Zlib+Base64 encoding for math data context"

### Phase 3: Update Python Render Template

- [ ] Open `vectomancy/templates/python.tera`.
- [ ] Add standard library imports at the top: `import json, zlib, base64, sys`.
- [ ] Replace the massive raw `strokes_data = ...` array with:
  ```python
  COMPRESSED_DATA = b"{{ encoded_data }}"

  def _load_data():
      try:
          raw_json = zlib.decompress(base64.b64decode(COMPRESSED_DATA)).decode('utf-8')
          return json.loads(raw_json)
      except Exception as e:
          print(f"Error decoding vector data: {e}", file=sys.stderr)
          sys.exit(1)

  strokes_data = _load_data()
  ```
- [ ] Remove any `# pylint: disable=all` or `# type: ignore` that were previously added to bypass AST explosion.
- [ ] Commit: "feat(templates): Update python template to decode Zlib+Base64 data"

### Phase 4: Integration and End-to-End Verification

- [ ] Run the CLI against a test image: `cargo run -- run tests/assets/miku.png -o /tmp/agents/vectomancy/miku_encoded.py -m spline -f python`
- [ ] Verify the file is created and its size is significantly smaller than the previous 359KB.
- [ ] Run the generated python script: `python /tmp/agents/vectomancy/miku_encoded.py` to ensure it successfully displays the plot.
- [ ] Commit: "test: Verify end-to-end encoded output and template rendering"
