use crate::cli::OutputFormat;
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

pub fn emit_file(
    ast: &MathExpressionAST,
    format: &OutputFormat,
    output_path: &Path,
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
            tera.add_template_file("templates/python.tera", Some("python"))?;
            "python"
        }
        OutputFormat::Latex => {
            tera.add_template_file("templates/latex.tera", Some("latex"))?;
            "latex"
        }
        OutputFormat::Html => {
            tera.add_template_file("templates/html.tera", Some("html"))?;
            "html"
        }
        OutputFormat::Geogebra => {
            tera.add_template_file("templates/geogebra.tera", Some("geogebra"))?;
            "geogebra"
        }
        OutputFormat::Wolfram => {
            tera.add_template_file("templates/wolfram.tera", Some("wolfram"))?;
            "wolfram"
        }
        OutputFormat::Json => unreachable!(),
    };

    let mut context = Context::new();
    match ast {
        MathExpressionAST::Fourier { strokes } => {
            let encoded = encode_math_data(strokes)?;
            context.insert("encoded_data", &encoded);
        }
        MathExpressionAST::Spline { equations } => {
            let encoded = encode_math_data(equations)?;
            context.insert("encoded_data", &encoded);
        }
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
