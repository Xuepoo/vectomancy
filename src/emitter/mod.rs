#[cfg(feature = "gpu")]
pub mod native;

use crate::config::OutputFormat;
use crate::error::VectomancyError;
use crate::models::MathExpressionAST;
use base64::Engine;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use tera::{Context, Tera};
use tracing::info;

pub fn encode_math_data<T: Serialize>(data: &T) -> Result<String, VectomancyError> {
    let json_str = serde_json::to_string(data)
        .map_err(|e| VectomancyError::InvalidInput(format!("JSON serialization error: {}", e)))?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(json_str.as_bytes())
        .map_err(|e| VectomancyError::InvalidInput(format!("Compression error: {}", e)))?;
    let compressed_bytes = encoder
        .finish()
        .map_err(|e| VectomancyError::InvalidInput(format!("Compression finish error: {}", e)))?;

    Ok(base64::engine::general_purpose::STANDARD.encode(compressed_bytes))
}

fn process_value_colors(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Object(map) => {
            if let Some(color_val) = map.get_mut("color_rgb") {
                if let Some(arr) = color_val.as_array() {
                    if arr.len() == 3 {
                        let r = (arr[0].as_f64().unwrap_or(0.0) * 255.0).round() as u8;
                        let g = (arr[1].as_f64().unwrap_or(0.0) * 255.0).round() as u8;
                        let b = (arr[2].as_f64().unwrap_or(0.0) * 255.0).round() as u8;
                        *color_val = serde_json::json!([r, g, b]);
                    }
                } else if let Some(obj) = color_val.as_object() {
                    let mut fallback = [0u8; 3];
                    if let Some(stops_val) = obj.get("stops").and_then(|s| s.as_array()) {
                        if let Some(first_stop) = stops_val.first().and_then(|s| s.as_array()) {
                            if first_stop.len() == 2 {
                                if let Some(rgb_arr) = first_stop[1].as_array() {
                                    if rgb_arr.len() == 3 {
                                        let r = (rgb_arr[0].as_f64().unwrap_or(0.0) * 255.0).round()
                                            as u8;
                                        let g = (rgb_arr[1].as_f64().unwrap_or(0.0) * 255.0).round()
                                            as u8;
                                        let b = (rgb_arr[2].as_f64().unwrap_or(0.0) * 255.0).round()
                                            as u8;
                                        fallback = [r, g, b];
                                    }
                                }
                            }
                        }
                    }
                    *color_val = serde_json::json!(fallback);
                }
            }
            for (_, v) in map.iter_mut() {
                process_value_colors(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                process_value_colors(v);
            }
        }
        _ => {}
    }
}

pub fn prepare_ast_for_template(
    ast: &MathExpressionAST,
) -> Result<serde_json::Value, VectomancyError> {
    let mut val = serde_json::to_value(ast)
        .map_err(|e| VectomancyError::InvalidInput(format!("JSON serialization error: {}", e)))?;
    process_value_colors(&mut val);
    Ok(val)
}

pub fn emit_file(
    ast: &MathExpressionAST,
    format: &OutputFormat,
    output_path: &Path,
    original_dimensions: (u32, u32),
) -> Result<(), VectomancyError> {
    info!("Initializing Tera template engine");

    // In a real application, you'd embed templates into the binary or read from XDG data dir.
    // We are reading relative to the current directory for simplicity in this scaffolding.
    let mut tera = Tera::default();

    if let OutputFormat::Json = format {
        info!("Serializing AST to JSON");
        let json_output = serde_json::to_string_pretty(ast).map_err(|e| {
            VectomancyError::InvalidInput(format!("JSON serialization error: {}", e))
        })?;
        info!("Writing output to {:?}", output_path);
        fs::write(output_path, json_output)?;
        return Ok(());
    }

    let template_name = match format {
        OutputFormat::Python => {
            tera.add_raw_template("python", include_str!("../../templates/python.tera"))?;
            "python"
        }
        OutputFormat::Html => {
            tera.add_raw_template("html", include_str!("../../templates/html.tera"))?;
            "html"
        }
        OutputFormat::Desmos => {
            tera.add_raw_template("desmos", include_str!("../../templates/desmos.tera"))?;
            "desmos"
        }
        OutputFormat::Json | OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp => {
            unreachable!()
        }
    };

    let processed_ast = prepare_ast_for_template(ast)?;
    let mut context = Context::new();
    match ast {
        MathExpressionAST::Fourier {
            strokes,
            bounding_box: _,
        } => {
            let encoded = encode_math_data(strokes)?;
            context.insert("encoded_data", &encoded);
            context.insert("is_fourier", &true);
            context.insert("strokes", &processed_ast["strokes"]);
        }
        MathExpressionAST::Spline {
            equations,
            bounding_box: _,
        } => {
            let encoded = encode_math_data(equations)?;
            context.insert("encoded_data", &encoded);
            context.insert("is_spline", &true);
            context.insert("equations", &processed_ast["equations"]);
        }
        MathExpressionAST::Polyline {
            paths,
            bounding_box: _,
        } => {
            let encoded = encode_math_data(paths)?;
            context.insert("encoded_data", &encoded);
            context.insert("is_polyline", &true);
            context.insert("paths", &processed_ast["paths"]);
        }
    }

    context.insert("width", &original_dimensions.0);
    context.insert("height", &original_dimensions.1);

    if let Some(file_stem) = output_path.file_stem() {
        context.insert("base_name", &file_stem.to_string_lossy());
    } else {
        context.insert("base_name", "output");
    }

    info!("Rendering template: {}", template_name);
    let rendered = tera.render(template_name, &context)?;

    info!("Writing output to {:?}", output_path);
    fs::write(output_path, rendered)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        value: String,
    }

    #[test]
    fn test_encode_math_data() {
        let data = TestData {
            value: "hello world".to_string(),
        };
        let encoded = encode_math_data(&data).unwrap();
        assert!(!encoded.is_empty());

        // Decode to verify
        use base64::Engine;
        use std::io::Read;
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
        let mut decoded_json = String::new();
        decoder.read_to_string(&mut decoded_json).unwrap();
        assert_eq!(decoded_json, r#"{"value":"hello world"}"#);
    }
}
