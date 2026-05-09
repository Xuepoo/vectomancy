# Design Spec: AST Explosion Fix via Base64/Zlib Embedding

**Date**: 2026-05-09
**Topic**: AST Explosion Fix

## 1. Problem Statement

Currently, `Vectomancy` extracts mathematical parameters (like Fourier coefficients or spline control points) and renders them as nested tuple/list expressions in target code formats (such as Python scripts).
For complex paths, this generates files comprising tens of thousands of lines of raw float lists. When opened in modern IDEs like VSCode or PyCharm, this triggers an "AST (Abstract Syntax Tree) Explosion," causing the language servers and editors to freeze, consume massive amounts of memory, or crash. It also results in unnecessarily large output files.

## 2. Proposed Solution

Instead of emitting raw code arrays, the `emitter` module in Rust will serialize the data, compress it, and pass a single base64-encoded string to the rendering templates. The templates will include a runtime decoding step to restore the native data structures.

This ensures the generated code consists of standard library imports, a single large string literal, and a few lines of decoding logic, which easily bypasses AST analysis overhead.

## 3. Architecture & Data Flow

### 3.1 Serialization & Compression (Rust)

- **Dependencies**: Add `serde_json` (if not already present), `flate2` (for Zlib compression), and `base64` to `Cargo.toml`.
- **Process**:
  1. The parsed/calculated math data (`strokes_data`) is serialized to a compact JSON string.
  2. The JSON string is compressed using Zlib (`flate2::write::ZlibEncoder`).
  3. The compressed byte array is encoded into a Base64 string (`base64::engine::general_purpose::STANDARD.encode`).
  4. The template context will receive `encoded_data` instead of the raw `strokes_data`.

### 3.2 Code Generation & Template Layer

- **Changes**: Update `vectomancy/templates/python.tera`.
- **Implementation**:

  ```python
  import json
  import zlib
  import base64
  import sys

  # ... other imports (matplotlib, numpy, etc.)

  COMPRESSED_DATA = b"{{ encoded_data }}"

  def _load_data():
      try:
          raw_json = zlib.decompress(base64.b64decode(COMPRESSED_DATA)).decode('utf-8')
          return json.loads(raw_json)
      except Exception as e:
          print(f"Error decoding vector data: {e}", file=sys.stderr)
          sys.exit(1)

  strokes_data = _load_data()
  # ... existing render logic uses strokes_data
  ```

## 4. Error Handling

- **Rust Side**: The serialization and compression process might fail (e.g., memory limits for extremely large paths, though unlikely). The `emitter` module should handle `serde_json::Error` and `std::io::Error` gracefully by wrapping them in the project's standard `Error` enum, preventing application panics.
- **Python Side**: If the decoding process fails (e.g., corrupted Base64 string due to template error, memory issues), a `try-except` block wraps the decoding, printing a clear error to `stderr` and exiting securely, avoiding deep tracebacks inside standard libraries.

## 5. Components Involved

- `vectomancy/src/emitter/mod.rs` (handling Tera context and data preparation).
- `vectomancy/Cargo.toml` (adding new crate dependencies).
- `vectomancy/templates/python.tera` (template rendering updates).

## 6. Testability & QA Scenarios

- **Unit Tests (Rust)**:
  - Add a test in `emitter/mod.rs` to verify that given a hardcoded `strokes_data` struct, the output `encoded_data` can be decoded via base64 -> decompress -> json, and matches the original struct.
- **Integration QA Scenario**:
  1. Execute `cargo run -- run tests/assets/miku.png -o /tmp/agents/vectomancy/miku_encoded.py -m spline -f python`.
  2. Verify that `/tmp/agents/vectomancy/miku_encoded.py` is generated.
  3. Verify the file size: `du -h /tmp/agents/vectomancy/miku_encoded.py` (should be < 100KB compared to 359KB previously).
  4. Execute the generated python script: `python /tmp/agents/vectomancy/miku_encoded.py` and ensure the plot window successfully opens with the image.
  5. Check AST explosion: Open the `.py` file in an IDE; there should be no syntax highlighting lag or CPU spikes.

## 7. Alternatives Considered

- **Pure JSON + Base64**: Simplest approach, but results in a larger output file size than Zlib compression.
- **Binary Array + Numpy (`frombuffer`)**: Fastest to parse in Python, but complicates the Rust serialization of nested arrays (list of lists of tuples) and limits cross-language compatibility if we want to add Javascript/HTML output later. JSON is highly cross-platform.
- **External Data File**: Saves data as a `.json` file alongside the `.py` script. Rejected because it breaks the "single self-contained script" convenience.

## 8. Success Criteria

- Generated `.py` file size is reduced by at least 70% compared to the old AST-heavy output.
- Opening the generated `.py` file in an IDE (e.g., VSCode) does not cause lag or high CPU usage from language servers.
- The Python script executes perfectly and generates the correct plots without requiring any non-standard libraries beyond `numpy` and `matplotlib`.
