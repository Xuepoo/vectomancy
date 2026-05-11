pub mod native;
pub mod scratch;

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
    if let OutputFormat::Scratch = format {
        info!("Emitting Scratch 3.0 project");
        return scratch::emit_scratch(ast, output_path, original_dimensions);
    }

    let template_name = match format {
        OutputFormat::Python => {
            tera.add_raw_template("python", include_str!("../../templates/python.tera"))?;
            "python"
        }
        OutputFormat::Latex => {
            tera.add_raw_template("latex", include_str!("../../templates/latex.tera"))?;
            "latex"
        }
        OutputFormat::Html => {
            tera.add_raw_template("html", include_str!("../../templates/html.tera"))?;
            "html"
        }
        OutputFormat::Geogebra => {
            tera.add_raw_template("geogebra", include_str!("../../templates/geogebra.tera"))?;
            "geogebra"
        }
        OutputFormat::Wolfram => {
            tera.add_raw_template("wolfram", include_str!("../../templates/wolfram.tera"))?;
            "wolfram"
        }
        OutputFormat::Kmplot => {
            tera.add_raw_template("kmplot", include_str!("../../templates/kmplot.tera"))?;
            "kmplot"
        }
        OutputFormat::Desmos => {
            tera.add_raw_template("desmos", include_str!("../../templates/desmos.tera"))?;
            "desmos"
        }
        OutputFormat::Scratch
        | OutputFormat::Json
        | OutputFormat::Png
        | OutputFormat::Jpg
        | OutputFormat::Webp => {
            unreachable!()
        }
    };

    let mut context = Context::new();
    match ast {
        MathExpressionAST::Fourier { strokes } => {
            let encoded = encode_math_data(strokes)?;
            context.insert("encoded_data", &encoded);
            context.insert("is_fourier", &true);
            context.insert("strokes", strokes);
        }
        MathExpressionAST::Spline { equations } => {
            let encoded = encode_math_data(equations)?;
            context.insert("encoded_data", &encoded);
            context.insert("is_spline", &true);
            context.insert("equations", equations);
        }
        MathExpressionAST::Polyline { paths } => {
            let encoded = encode_math_data(paths)?;
            context.insert("encoded_data", &encoded);
            context.insert("is_polyline", &true);
            context.insert("paths", paths);
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
    if let OutputFormat::Geogebra = format {
        let file = std::fs::File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("geogebra.xml", options)
            .map_err(|e| VectomancyError::InvalidInput(format!("Zip error: {}", e)))?;
        zip.write_all(rendered.as_bytes())
            .map_err(|e| VectomancyError::InvalidInput(format!("Zip write error: {}", e)))?;
        zip.finish()
            .map_err(|e| VectomancyError::InvalidInput(format!("Zip finish error: {}", e)))?;
    } else {
        fs::write(output_path, rendered)?;
    }

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
