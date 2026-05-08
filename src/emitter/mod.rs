use crate::cli::OutputFormat;
use crate::error::VectomancyError;
use crate::models::MathExpressionAST;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};
use tracing::info;

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
        OutputFormat::Json => unreachable!(),
    };

    let mut context = Context::new();
    match ast {
        MathExpressionAST::Fourier { strokes } => {
            context.insert("strokes", strokes);
        }
        MathExpressionAST::Spline { equations } => {
            context.insert("equations", equations);
        }
    }

    info!("Rendering template: {}", template_name);
    let rendered = tera.render(template_name, &context)?;

    info!("Writing output to {:?}", output_path);
    fs::write(output_path, rendered)?;

    Ok(())
}
