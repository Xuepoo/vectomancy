#[cfg(feature = "gpu")]
pub mod native;
pub mod svg;

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

pub(super) fn validate_finite_ast(ast: &MathExpressionAST) -> Result<(), VectomancyError> {
    let finite = |value: f64, name: &str| {
        if value.is_finite() {
            Ok(())
        } else {
            Err(VectomancyError::InvalidInput(format!(
                "Non-finite {name} in math expression AST"
            )))
        }
    };

    let validate_bbox = |bbox: &[f32; 4]| {
        for value in bbox {
            if !value.is_finite() {
                return Err(VectomancyError::InvalidInput(
                    "Non-finite bounding box in math expression AST".to_string(),
                ));
            }
        }
        if bbox[2] < bbox[0] || bbox[3] < bbox[1] {
            return Err(VectomancyError::InvalidInput(
                "Bounding box maximum must not be less than minimum".to_string(),
            ));
        }
        let width = f64::from(bbox[2]) - f64::from(bbox[0]);
        let height = f64::from(bbox[3]) - f64::from(bbox[1]);
        if !width.is_finite() || !height.is_finite() {
            return Err(VectomancyError::InvalidInput(
                "Non-finite bounding box extent in math expression AST".to_string(),
            ));
        }
        Ok(())
    };

    let validate_color = |style: &Option<crate::models::ColorStyle>| {
        if let Some(style) = style {
            let finite = |value: f32| value.is_finite();
            match style {
                crate::models::ColorStyle::Solid(rgb) => {
                    if rgb.iter().copied().any(|value| !finite(value)) {
                        return Err(VectomancyError::InvalidInput(
                            "Non-finite solid color in math expression AST".to_string(),
                        ));
                    }
                }
                crate::models::ColorStyle::LinearGradient(gradient) => {
                    if gradient.stops.iter().any(|(offset, rgb)| {
                        !finite(*offset) || rgb.iter().copied().any(|v| !finite(v))
                    }) || gradient.start_pos.iter().copied().any(|v| !finite(v))
                        || gradient.end_pos.iter().copied().any(|v| !finite(v))
                    {
                        return Err(VectomancyError::InvalidInput(
                            "Non-finite gradient color in math expression AST".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    };

    match ast {
        MathExpressionAST::Fourier {
            strokes,
            bounding_box,
        } => {
            validate_bbox(bounding_box)?;
            for stroke in strokes {
                validate_color(&stroke.color_style)?;
                for term in &stroke.data {
                    finite(term.amplitude, "Fourier amplitude")?;
                    finite(term.frequency, "Fourier frequency")?;
                    finite(term.phase, "Fourier phase")?;
                }
            }
        }
        MathExpressionAST::Spline {
            equations,
            bounding_box,
        } => {
            validate_bbox(bounding_box)?;
            for path in equations {
                validate_color(&path.color_style)?;
                let mut previous_end = None;
                for equation in &path.data {
                    finite(equation.start_t, "spline start_t")?;
                    finite(equation.end_t, "spline end_t")?;
                    if equation.end_t < equation.start_t {
                        return Err(VectomancyError::InvalidInput(
                            "Spline end_t must not be less than start_t".to_string(),
                        ));
                    }
                    if previous_end.is_some_and(|end| equation.start_t < end) {
                        return Err(VectomancyError::InvalidInput(
                            "Spline intervals must be ordered and non-overlapping".to_string(),
                        ));
                    }
                    previous_end = Some(equation.end_t);
                    for coefficient in equation.x_poly.iter().chain(&equation.y_poly) {
                        finite(*coefficient, "spline coefficient")?;
                    }
                }
            }
        }
        MathExpressionAST::Polyline {
            paths,
            bounding_box,
        } => {
            validate_bbox(bounding_box)?;
            for path in paths {
                validate_color(&path.color_style)?;
                for point in &path.data {
                    finite(point.x, "polyline x coordinate")?;
                    finite(point.y, "polyline y coordinate")?;
                }
            }
        }
    }
    Ok(())
}

fn fmt_expression_number(value: f64) -> String {
    if value == 0.0 {
        "0".to_string()
    } else {
        value.to_string()
    }
}

fn fmt_polynomial(coefficients: &[f64], variable: &str, javascript: bool) -> String {
    let mut terms = Vec::new();
    for (power, &coefficient) in coefficients.iter().enumerate() {
        if coefficient == 0.0 {
            continue;
        }

        let magnitude = coefficient.abs();
        let coefficient_text = if power == 0 || magnitude != 1.0 {
            fmt_expression_number(magnitude)
        } else {
            String::new()
        };
        let variable_text = if power == 0 {
            String::new()
        } else if power == 1 {
            variable.to_string()
        } else if javascript {
            format!("Math.pow({variable}, {power})")
        } else {
            format!("{variable}^{power}")
        };
        let body = match (coefficient_text.is_empty(), variable_text.is_empty()) {
            (true, true) => "1".to_string(),
            (true, false) => variable_text,
            (false, true) => coefficient_text,
            (false, false) => format!("{coefficient_text}*{variable_text}"),
        };
        terms.push(if coefficient.is_sign_negative() {
            format!("-{body}")
        } else {
            body
        });
    }

    if terms.is_empty() {
        "0".to_string()
    } else {
        let mut expression = terms[0].clone();
        for term in terms.iter().skip(1) {
            if let Some(positive) = term.strip_prefix('-') {
                expression.push_str(" - ");
                expression.push_str(positive);
            } else {
                expression.push_str(" + ");
                expression.push_str(term);
            }
        }
        expression
    }
}

fn add_spline_expressions(value: &mut serde_json::Value) {
    let Some(paths) = value.get_mut("equations").and_then(|v| v.as_array_mut()) else {
        return;
    };
    for path in paths {
        let Some(object) = path.as_object_mut() else {
            continue;
        };
        let domain = object
            .get("data")
            .and_then(serde_json::Value::as_array)
            .and_then(|equations| Some((equations.first()?, equations.last()?)))
            .map(|(first, last)| {
                (
                    first.get("start_t").cloned().unwrap_or_default(),
                    last.get("end_t").cloned().unwrap_or_default(),
                )
            });
        if let Some((domain_start, domain_end)) = domain {
            object.insert("domain_start".to_string(), domain_start);
            object.insert("domain_end".to_string(), domain_end);
        }
        let Some(equations) = object
            .get_mut("data")
            .and_then(serde_json::Value::as_array_mut)
        else {
            continue;
        };
        for equation in equations {
            let Some(object) = equation.as_object_mut() else {
                continue;
            };
            let start = object
                .get("start_t")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            let variable = if start == 0.0 {
                "t".to_string()
            } else if start.is_sign_negative() {
                format!("(t+{})", fmt_expression_number(start.abs()))
            } else {
                format!("(t-{})", fmt_expression_number(start))
            };
            let x_poly = object
                .get("x_poly")
                .and_then(serde_json::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(serde_json::Value::as_f64)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let y_poly = object
                .get("y_poly")
                .and_then(serde_json::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(serde_json::Value::as_f64)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            object.insert(
                "x_expression".to_string(),
                serde_json::Value::String(fmt_polynomial(&x_poly, &variable, false)),
            );
            object.insert(
                "y_expression".to_string(),
                serde_json::Value::String(fmt_polynomial(&y_poly, &variable, false)),
            );
            object.insert(
                "x_expression_js".to_string(),
                serde_json::Value::String(fmt_polynomial(&x_poly, &variable, true)),
            );
            object.insert(
                "y_expression_js".to_string(),
                serde_json::Value::String(fmt_polynomial(&y_poly, &variable, true)),
            );
        }
    }
}

fn fmt_fourier_argument(frequency: f64, phase: f64) -> String {
    let frequency_term = if frequency == 0.0 {
        String::new()
    } else if frequency == 1.0 {
        "t".to_string()
    } else if frequency == -1.0 {
        "-t".to_string()
    } else {
        format!("{}*t", fmt_expression_number(frequency))
    };

    if frequency_term.is_empty() {
        fmt_expression_number(phase)
    } else if phase == 0.0 {
        frequency_term
    } else if phase.is_sign_negative() {
        format!("{frequency_term} - {}", fmt_expression_number(phase.abs()))
    } else {
        format!("{frequency_term} + {}", fmt_expression_number(phase))
    }
}

fn fmt_fourier_expression(terms: &[serde_json::Value], function: &str, javascript: bool) -> String {
    let mut expressions = Vec::new();
    for term in terms {
        let amplitude = term
            .get("amplitude")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        if amplitude == 0.0 {
            continue;
        }
        let frequency = term
            .get("frequency")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let phase = term
            .get("phase")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let argument = fmt_fourier_argument(frequency, phase);
        let function_call = if javascript {
            format!("Math.{function}({argument})")
        } else {
            format!("\\{function}({argument})")
        };
        let magnitude = amplitude.abs();
        let body = if magnitude == 1.0 {
            function_call
        } else {
            format!("{}*{function_call}", fmt_expression_number(magnitude))
        };
        expressions.push(if amplitude.is_sign_negative() {
            format!("-{body}")
        } else {
            body
        });
    }

    if expressions.is_empty() {
        "0".to_string()
    } else {
        let mut expression = expressions[0].clone();
        for term in expressions.iter().skip(1) {
            if let Some(positive) = term.strip_prefix('-') {
                expression.push_str(" - ");
                expression.push_str(positive);
            } else {
                expression.push_str(" + ");
                expression.push_str(term);
            }
        }
        expression
    }
}

fn add_fourier_expressions(value: &mut serde_json::Value) {
    let Some(strokes) = value
        .get_mut("strokes")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return;
    };
    for stroke in strokes {
        let Some(object) = stroke.as_object_mut() else {
            continue;
        };
        let Some(terms) = object.get("data").and_then(serde_json::Value::as_array) else {
            continue;
        };
        let x_expression = fmt_fourier_expression(terms, "cos", false);
        let y_expression = fmt_fourier_expression(terms, "sin", false);
        let x_expression_js = fmt_fourier_expression(terms, "cos", true);
        let y_expression_js = fmt_fourier_expression(terms, "sin", true);
        object.insert(
            "x_expression".to_string(),
            serde_json::Value::String(x_expression),
        );
        object.insert(
            "y_expression".to_string(),
            serde_json::Value::String(y_expression),
        );
        object.insert(
            "x_expression_js".to_string(),
            serde_json::Value::String(x_expression_js),
        );
        object.insert(
            "y_expression_js".to_string(),
            serde_json::Value::String(y_expression_js),
        );
    }
}

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
    validate_finite_ast(ast)?;
    let mut val = serde_json::to_value(ast)
        .map_err(|e| VectomancyError::InvalidInput(format!("JSON serialization error: {}", e)))?;
    process_value_colors(&mut val);
    add_spline_expressions(&mut val);
    add_fourier_expressions(&mut val);
    Ok(val)
}

pub fn emit_file(
    ast: &MathExpressionAST,
    format: &OutputFormat,
    output_path: &Path,
    original_dimensions: (u32, u32),
    stroke_width: f32,
) -> Result<(), VectomancyError> {
    info!("Initializing Tera template engine");
    validate_finite_ast(ast)?;

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

    if let OutputFormat::Svg = format {
        info!("Rendering AST to SVG");
        let svg_output = svg::to_svg_string(ast, original_dimensions, stroke_width)?;
        info!("Writing output to {:?}", output_path);
        fs::write(output_path, svg_output)?;
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
        OutputFormat::Json
        | OutputFormat::Svg
        | OutputFormat::Png
        | OutputFormat::Jpg
        | OutputFormat::Webp => {
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
    let bounding_box = match ast {
        MathExpressionAST::Fourier { bounding_box, .. }
        | MathExpressionAST::Spline { bounding_box, .. }
        | MathExpressionAST::Polyline { bounding_box, .. } => bounding_box,
    };
    context.insert("bbox_min_x", &bounding_box[0]);
    context.insert("bbox_min_y", &bounding_box[1]);
    context.insert("bbox_max_x", &bounding_box[2]);
    context.insert("bbox_max_y", &bounding_box[3]);
    let viewport_width = (f64::from(bounding_box[2]) - f64::from(bounding_box[0])).max(1.0);
    let viewport_height = (f64::from(bounding_box[3]) - f64::from(bounding_box[1])).max(1.0);
    context.insert("viewport_width", &viewport_width);
    context.insert("viewport_height", &viewport_height);
    context.insert("has_x_offset", &(bounding_box[0] != 0.0));
    context.insert("has_y_offset", &(bounding_box[1] != 0.0));

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

    #[test]
    fn polynomial_formatter_omits_zero_terms_and_normalizes_signs() {
        assert_eq!(
            fmt_polynomial(&[-0.0, 2.0, 0.0, -1.0], "(t-3)", false),
            "2*(t-3) - (t-3)^3"
        );
        assert_eq!(fmt_polynomial(&[0.0, 0.0], "t", true), "0");
        assert_eq!(fmt_expression_number(-0.0), "0");
    }

    #[test]
    fn fourier_formatter_omits_zero_and_identity_operations() {
        let terms = serde_json::json!([
            {"amplitude": 0.0, "frequency": 8.0, "phase": 2.0},
            {"amplitude": 1.0, "frequency": 1.0, "phase": -0.0},
            {"amplitude": -2.0, "frequency": -1.0, "phase": 3.0}
        ]);
        assert_eq!(
            fmt_fourier_expression(terms.as_array().unwrap(), "cos", false),
            "\\cos(t) - 2*\\cos(-t + 3)"
        );
        assert_eq!(
            fmt_fourier_expression(terms.as_array().unwrap(), "sin", true),
            "Math.sin(t) - 2*Math.sin(-t + 3)"
        );
    }

    #[test]
    fn prepared_spline_contains_renderable_expressions() {
        let ast = MathExpressionAST::Spline {
            equations: vec![crate::models::ColoredPath {
                color_style: None,
                data: vec![crate::models::SplineEquation {
                    start_t: 2.0,
                    end_t: 3.0,
                    x_poly: vec![10.0, 1.0, 0.0, -0.5],
                    y_poly: vec![0.0, 0.0, 0.0, 0.0],
                }],
            }],
            bounding_box: [0.0, 0.0, 10.0, 10.0],
        };
        let value = prepare_ast_for_template(&ast).unwrap();
        let equation = &value["equations"][0]["data"][0];
        assert_eq!(equation["x_expression"], "10 + (t-2) - 0.5*(t-2)^3");
        assert_eq!(
            equation["x_expression_js"],
            "10 + (t-2) - 0.5*Math.pow((t-2), 3)"
        );
        assert_eq!(equation["y_expression"], "0");
    }

    #[test]
    fn desmos_template_uses_valid_compact_spline_expressions_and_bounds() {
        let ast = MathExpressionAST::Spline {
            equations: vec![crate::models::ColoredPath {
                color_style: None,
                data: vec![crate::models::SplineEquation {
                    start_t: 0.0,
                    end_t: 1.0,
                    x_poly: vec![10.0, 1.0, 0.0, -0.5],
                    y_poly: vec![-0.0, 0.0, 0.0, 0.0],
                }],
            }],
            bounding_box: [0.0, 0.0, 20.0, 30.0],
        };
        let dir = std::env::temp_dir();
        let output = dir.join(format!("vectomancy-desmos-{}.html", std::process::id()));
        emit_file(&ast, &OutputFormat::Desmos, &output, (20, 30), 1.0).unwrap();
        let rendered = std::fs::read_to_string(&output).unwrap();
        let _ = std::fs::remove_file(output);

        assert!(rendered.contains("'0 \\\\le t \\\\le 1: 10 + t - 0.5*t^3'"));
        assert!(rendered.contains("right: 20"));
        assert!(rendered.contains("top: 30"));
        assert!(!rendered.contains("t-0"));
        assert!(!rendered.contains("+ -0"));
        assert!(!rendered.contains("*0"));
    }

    #[test]
    fn desmos_template_uses_spline_parameter_domain() {
        let ast = MathExpressionAST::Spline {
            equations: vec![crate::models::ColoredPath {
                color_style: None,
                data: vec![crate::models::SplineEquation {
                    start_t: 2.0,
                    end_t: 3.0,
                    x_poly: vec![10.0, 1.0],
                    y_poly: vec![20.0, 1.0],
                }],
            }],
            bounding_box: [0.0, 0.0, 30.0, 40.0],
        };
        let output = std::env::temp_dir().join(format!(
            "vectomancy-desmos-domain-{}.html",
            std::process::id()
        ));
        emit_file(&ast, &OutputFormat::Desmos, &output, (30, 40), 1.0).unwrap();
        let rendered = std::fs::read_to_string(&output).unwrap();
        let _ = std::fs::remove_file(output);

        assert!(rendered.contains("parametricDomain: {min: '2', max: '3'}"));
    }

    #[test]
    fn rejects_non_finite_template_values() {
        let ast = MathExpressionAST::Spline {
            equations: vec![crate::models::ColoredPath {
                color_style: None,
                data: vec![crate::models::SplineEquation {
                    start_t: 0.0,
                    end_t: 1.0,
                    x_poly: vec![1.0, f64::NAN],
                    y_poly: vec![0.0, 1.0],
                }],
            }],
            bounding_box: [0.0, 0.0, 10.0, 10.0],
        };

        assert!(prepare_ast_for_template(&ast).is_err());
    }

    #[test]
    fn rejects_out_of_order_spline_intervals() {
        let ast = MathExpressionAST::Spline {
            equations: vec![crate::models::ColoredPath {
                color_style: None,
                data: vec![
                    crate::models::SplineEquation {
                        start_t: 2.0,
                        end_t: 3.0,
                        x_poly: vec![0.0],
                        y_poly: vec![0.0],
                    },
                    crate::models::SplineEquation {
                        start_t: 1.0,
                        end_t: 2.0,
                        x_poly: vec![0.0],
                        y_poly: vec![0.0],
                    },
                ],
            }],
            bounding_box: [0.0, 0.0, 1.0, 1.0],
        };

        assert!(validate_finite_ast(&ast).is_err());
    }

    #[test]
    fn json_export_rejects_non_finite_values() {
        let ast = MathExpressionAST::Polyline {
            paths: vec![crate::models::ColoredPath {
                color_style: None,
                data: vec![crate::models::Point2D {
                    x: f64::NAN,
                    y: 0.0,
                }],
            }],
            bounding_box: [0.0, 0.0, 1.0, 1.0],
        };
        let output = std::env::temp_dir().join(format!(
            "vectomancy-invalid-json-{}.json",
            std::process::id()
        ));

        assert!(emit_file(&ast, &OutputFormat::Json, &output, (1, 1), 1.0).is_err());
        assert!(!output.exists());
    }

    #[test]
    fn html_template_uses_bounded_index_sampling() {
        let ast = MathExpressionAST::Spline {
            equations: vec![crate::models::ColoredPath {
                color_style: None,
                data: vec![crate::models::SplineEquation {
                    start_t: 2.0,
                    end_t: 3.0,
                    x_poly: vec![10.0, 1.0],
                    y_poly: vec![20.0, 1.0],
                }],
            }],
            bounding_box: [10.0, 20.0, 11.0, 21.0],
        };
        let output = std::env::temp_dir().join(format!(
            "vectomancy-html-sampling-{}.html",
            std::process::id()
        ));
        emit_file(&ast, &OutputFormat::Html, &output, (1, 1), 1.0).unwrap();
        let rendered = std::fs::read_to_string(&output).unwrap();
        let _ = std::fs::remove_file(output);

        assert!(rendered.contains("Math.min(10000"));
        assert!(rendered.contains("for (let i = 0; i <= steps; i++)"));
        assert!(rendered.contains("const t = 2 + (3 - 2) * i / steps"));
        assert!(rendered.contains("let x = 10 + (t-2) - 10;"));
    }
}
