use crate::error::VectomancyError;
use crate::models::{ColorStyle, ColoredPath, MathExpressionAST};
use std::fmt::Write as _;

/// Number of samples per Fourier stroke period, matching native raster tessellation density.
const FOURIER_STEPS: usize = 720;
/// Minimum samples for each cycle of the highest represented frequency.
const FOURIER_SAMPLES_PER_CYCLE: usize = 16;
/// Prevent pathological frequency values from producing unbounded SVG output.
const MAX_FOURIER_STEPS: usize = 1_000_000;
/// Number of samples per spline segment when flattening to a polyline for the `<path>` `d` attribute.
const SPLINE_STEPS: usize = 64;

fn fmt_num(v: f64) -> Result<String, VectomancyError> {
    if !v.is_finite() {
        return Err(VectomancyError::InvalidInput(
            "Non-finite coordinate while rendering SVG".to_string(),
        ));
    }
    // Trim to 4 decimal places; drop trailing zeros to keep markup compact.
    let normalized = if v.abs() < 0.00005 { 0.0 } else { v };
    let mut s = format!("{normalized:.4}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    Ok(s)
}

fn escape_id(idx: usize) -> String {
    format!("vecto-grad-{}", idx)
}

/// Renders a `ColorStyle` as either a `stroke="rgb(...)"` attribute value or a `url(#id)` reference,
/// appending any required `<linearGradient>` definitions to `defs`.
fn resolve_stroke(
    style: &Option<ColorStyle>,
    bbox: [f32; 4],
    defs: &mut String,
    next_grad_id: &mut usize,
) -> Result<String, VectomancyError> {
    match style {
        None => Ok("#000000".to_string()),
        Some(ColorStyle::Solid(rgb)) => {
            let r = (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8;
            Ok(format!("rgb({}, {}, {})", r, g, b))
        }
        Some(ColorStyle::LinearGradient(grad)) => {
            let id = escape_id(*next_grad_id);
            *next_grad_id += 1;

            let w = (bbox[2] - bbox[0]) as f64;
            let h = (bbox[3] - bbox[1]) as f64;
            let b0 = bbox[0] as f64;
            let b1 = bbox[1] as f64;

            let x0 = b0 + grad.start_pos[0] as f64 * w;
            let y0 = b1 + grad.start_pos[1] as f64 * h;
            let x1 = b0 + grad.end_pos[0] as f64 * w;
            let y1 = b1 + grad.end_pos[1] as f64 * h;

            let _ = write!(
                defs,
                r#"<linearGradient id="{id}" gradientUnits="userSpaceOnUse" x1="{x0}" y1="{y0}" x2="{x1}" y2="{y1}">"#,
                id = id,
                x0 = fmt_num(x0)?,
                y0 = fmt_num(y0)?,
                x1 = fmt_num(x1)?,
                y1 = fmt_num(y1)?,
            );
            for (offset, rgb) in &grad.stops {
                let r = (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8;
                let g = (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8;
                let b = (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8;
                let _ = write!(
                    defs,
                    r#"<stop offset="{}" stop-color="rgb({}, {}, {})"/>"#,
                    fmt_num(offset.clamp(0.0, 1.0) as f64)?,
                    r,
                    g,
                    b
                );
            }
            defs.push_str("</linearGradient>");

            Ok(format!("url(#{})", id))
        }
    }
}

fn polyline_path(points: impl Iterator<Item = (f64, f64)>) -> Result<String, VectomancyError> {
    let mut d = String::new();
    for (i, (x, y)) in points.enumerate() {
        if i == 0 {
            let _ = write!(d, "M {} {}", fmt_num(x)?, fmt_num(y)?);
        } else {
            let _ = write!(d, " L {} {}", fmt_num(x)?, fmt_num(y)?);
        }
    }
    Ok(d)
}

fn emit_polyline_paths(
    paths: &[ColoredPath<Vec<crate::models::Point2D>>],
    bbox: [f32; 4],
    defs: &mut String,
    next_grad_id: &mut usize,
    body: &mut String,
    stroke_width: f32,
) -> Result<(), VectomancyError> {
    for path in paths {
        if path.data.is_empty() {
            continue;
        }
        let stroke = resolve_stroke(&path.color_style, bbox, defs, next_grad_id)?;
        let d = polyline_path(path.data.iter().map(|p| (p.x, p.y)))?;
        let _ = write!(
            body,
            r#"<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{}"/>"#,
            stroke_width
        );
    }
    Ok(())
}

fn emit_spline_paths(
    equations: &[ColoredPath<Vec<crate::models::SplineEquation>>],
    bbox: [f32; 4],
    defs: &mut String,
    next_grad_id: &mut usize,
    body: &mut String,
    stroke_width: f32,
) -> Result<(), VectomancyError> {
    for path in equations {
        if path.data.is_empty() {
            continue;
        }
        let stroke = resolve_stroke(&path.color_style, bbox, defs, next_grad_id)?;
        let mut points = Vec::new();
        for eq in &path.data {
            for i in 0..=SPLINE_STEPS {
                let t = eq.start_t + (eq.end_t - eq.start_t) * i as f64 / SPLINE_STEPS as f64;
                let local_t = t - eq.start_t;
                let mut x = 0.0;
                let mut y = 0.0;
                for (j, coef) in eq.x_poly.iter().enumerate() {
                    x += coef * local_t.powi(j as i32);
                }
                for (j, coef) in eq.y_poly.iter().enumerate() {
                    y += coef * local_t.powi(j as i32);
                }
                points.push((x, y));
            }
        }
        let d = polyline_path(points.into_iter())?;
        let _ = write!(
            body,
            r#"<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{}"/>"#,
            stroke_width
        );
    }
    Ok(())
}

fn emit_fourier_paths(
    strokes: &[ColoredPath<Vec<crate::models::FourierTerm>>],
    bbox: [f32; 4],
    defs: &mut String,
    next_grad_id: &mut usize,
    body: &mut String,
    stroke_width: f32,
) -> Result<(), VectomancyError> {
    for path in strokes {
        if path.data.is_empty() {
            continue;
        }
        let stroke = resolve_stroke(&path.color_style, bbox, defs, next_grad_id)?;
        let max_frequency = path
            .data
            .iter()
            .filter(|term| term.amplitude != 0.0)
            .map(|term| term.frequency.abs())
            .fold(0.0, f64::max);
        let required_steps = (FOURIER_SAMPLES_PER_CYCLE as f64 * max_frequency)
            .ceil()
            .max(FOURIER_STEPS as f64);
        if required_steps > MAX_FOURIER_STEPS as f64 {
            return Err(VectomancyError::InvalidInput(format!(
                "Fourier frequency requires more than {MAX_FOURIER_STEPS} SVG samples"
            )));
        }
        let steps = required_steps as usize;
        let mut points = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let t = i as f64 / steps as f64 * std::f64::consts::TAU;
            let mut x = 0.0;
            let mut y = 0.0;
            for term in &path.data {
                let angle = term.frequency * t + term.phase;
                x += term.amplitude * angle.cos();
                y += term.amplitude * angle.sin();
            }
            points.push((x, y));
        }
        let d = polyline_path(points.into_iter())?;
        let _ = write!(
            body,
            r#"<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{}"/>"#,
            stroke_width
        );
    }
    Ok(())
}

/// Renders a `MathExpressionAST` into a standalone, viewBox-scoped SVG document.
///
/// Curves are flattened to polylines (`<path d="M...L...">`) rather than emitted as
/// exact Bezier/parametric commands, since Fourier epicycles have no closed-form SVG
/// representation and a single flattening strategy keeps Spline/Fourier/Polyline output
/// visually consistent. Colors reuse the same `ColorStyle` model as native rendering,
/// with linear gradients lowered to `<linearGradient>` defs.
pub fn to_svg_string(
    ast: &MathExpressionAST,
    original_dimensions: (u32, u32),
    stroke_width: f32,
) -> Result<String, VectomancyError> {
    super::validate_finite_ast(ast)?;
    if !stroke_width.is_finite() {
        return Err(VectomancyError::InvalidInput(
            "Non-finite SVG stroke width".to_string(),
        ));
    }
    let bbox = match ast {
        MathExpressionAST::Fourier { bounding_box, .. } => *bounding_box,
        MathExpressionAST::Spline { bounding_box, .. } => *bounding_box,
        MathExpressionAST::Polyline { bounding_box, .. } => *bounding_box,
    };

    let mut defs = String::new();
    let mut body = String::new();
    let mut next_grad_id = 0usize;

    match ast {
        MathExpressionAST::Fourier { strokes, .. } => {
            emit_fourier_paths(
                strokes,
                bbox,
                &mut defs,
                &mut next_grad_id,
                &mut body,
                stroke_width,
            )?;
        }
        MathExpressionAST::Spline { equations, .. } => {
            emit_spline_paths(
                equations,
                bbox,
                &mut defs,
                &mut next_grad_id,
                &mut body,
                stroke_width,
            )?;
        }
        MathExpressionAST::Polyline { paths, .. } => {
            emit_polyline_paths(
                paths,
                bbox,
                &mut defs,
                &mut next_grad_id,
                &mut body,
                stroke_width,
            )?;
        }
    }

    let (width, height) = original_dimensions;
    let view_x = fmt_num(bbox[0] as f64)?;
    let view_y = fmt_num(bbox[1] as f64)?;
    let view_width = fmt_num((f64::from(bbox[2]) - f64::from(bbox[0])).max(1.0))?;
    let view_height = fmt_num((f64::from(bbox[3]) - f64::from(bbox[1])).max(1.0))?;
    let mut out = String::new();
    let _ = write!(
        out,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="{view_x} {view_y} {view_width} {view_height}">
"#,
        width = width,
        height = height,
        view_x = view_x,
        view_y = view_y,
        view_width = view_width,
        view_height = view_height,
    );
    if !defs.is_empty() {
        let _ = writeln!(out, "<defs>{}</defs>", defs);
    }
    out.push_str(&body);
    out.push_str("\n</svg>\n");

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ColoredPath, FourierTerm, Point2D, SplineEquation};

    #[test]
    fn polyline_renders_moveto_lineto() {
        let ast = MathExpressionAST::Polyline {
            paths: vec![ColoredPath {
                color_style: Some(ColorStyle::Solid([1.0, 0.0, 0.0])),
                data: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: 10.0, y: 5.0 }],
            }],
            bounding_box: [0.0, 0.0, 10.0, 5.0],
        };
        let svg = to_svg_string(&ast, (10, 5), 1.0).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("M 0 0 L 10 5"));
        assert!(svg.contains("rgb(255, 0, 0)"));
    }

    #[test]
    fn number_formatter_normalizes_negative_zero() {
        assert_eq!(fmt_num(-0.00001).unwrap(), "0");
    }

    #[test]
    fn rejects_non_finite_coordinates() {
        assert!(fmt_num(f64::NAN).is_err());
        assert!(fmt_num(f64::INFINITY).is_err());
    }

    #[test]
    fn spline_uses_full_parameter_interval() {
        let ast = MathExpressionAST::Spline {
            equations: vec![ColoredPath {
                color_style: None,
                data: vec![SplineEquation {
                    start_t: 2.0,
                    end_t: 4.0,
                    x_poly: vec![0.0, 1.0],
                    y_poly: vec![0.0],
                }],
            }],
            bounding_box: [0.0, 0.0, 2.0, 1.0],
        };

        let svg = to_svg_string(&ast, (2, 1), 1.0).unwrap();
        assert!(svg.contains("L 2 0"));
    }

    #[test]
    fn high_frequency_fourier_does_not_alias_to_a_point() {
        let ast = MathExpressionAST::Fourier {
            strokes: vec![ColoredPath {
                color_style: None,
                data: vec![FourierTerm {
                    amplitude: 10.0,
                    frequency: 720.0,
                    phase: 0.0,
                }],
            }],
            bounding_box: [-10.0, -10.0, 10.0, 10.0],
        };

        let svg = to_svg_string(&ast, (20, 20), 1.0).unwrap();
        assert!(svg.contains("L -10 0"));
    }

    #[test]
    fn zero_amplitude_frequency_does_not_raise_sampling_limit() {
        let ast = MathExpressionAST::Fourier {
            strokes: vec![ColoredPath {
                color_style: None,
                data: vec![
                    FourierTerm {
                        amplitude: 0.0,
                        frequency: 1_000_000.0,
                        phase: 0.0,
                    },
                    FourierTerm {
                        amplitude: 1.0,
                        frequency: 1.0,
                        phase: 0.0,
                    },
                ],
            }],
            bounding_box: [-1.0, -1.0, 1.0, 1.0],
        };

        assert!(to_svg_string(&ast, (2, 2), 1.0).is_ok());
    }

    #[test]
    fn rejects_excessive_nonzero_fourier_sampling() {
        let ast = MathExpressionAST::Fourier {
            strokes: vec![ColoredPath {
                color_style: None,
                data: vec![FourierTerm {
                    amplitude: 1.0,
                    frequency: 100_000.0,
                    phase: 0.0,
                }],
            }],
            bounding_box: [-1.0, -1.0, 1.0, 1.0],
        };

        assert!(to_svg_string(&ast, (2, 2), 1.0).is_err());
    }

    #[test]
    fn gradient_emits_linear_gradient_def() {
        let grad = crate::models::GradientData {
            stops: vec![(0.0, [1.0, 0.0, 0.0]), (1.0, [0.0, 0.0, 1.0])],
            start_pos: [0.0, 0.5],
            end_pos: [1.0, 0.5],
        };
        let ast = MathExpressionAST::Polyline {
            paths: vec![ColoredPath {
                color_style: Some(ColorStyle::LinearGradient(std::sync::Arc::new(grad))),
                data: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: 10.0, y: 0.0 }],
            }],
            bounding_box: [0.0, 0.0, 10.0, 10.0],
        };
        let svg = to_svg_string(&ast, (10, 10), 1.0).unwrap();
        assert!(svg.contains("<linearGradient"));
        assert!(svg.contains("url(#vecto-grad-0)"));
    }

    #[test]
    fn fourier_and_spline_paths_are_nonempty() {
        let ast = MathExpressionAST::Fourier {
            strokes: vec![ColoredPath {
                color_style: None,
                data: vec![FourierTerm {
                    amplitude: 5.0,
                    frequency: 1.0,
                    phase: 0.0,
                }],
            }],
            bounding_box: [-5.0, -5.0, 5.0, 5.0],
        };
        let svg = to_svg_string(&ast, (10, 10), 1.0).unwrap();
        assert!(svg.contains("<path d=\"M"));

        let ast2 = MathExpressionAST::Spline {
            equations: vec![ColoredPath {
                color_style: None,
                data: vec![SplineEquation {
                    start_t: 0.0,
                    end_t: 1.0,
                    x_poly: vec![0.0, 10.0, 0.0, 0.0],
                    y_poly: vec![0.0, 0.0, 0.0, 0.0],
                }],
            }],
            bounding_box: [0.0, 0.0, 10.0, 0.0],
        };
        let svg2 = to_svg_string(&ast2, (10, 10), 1.0).unwrap();
        assert!(svg2.contains("<path d=\"M"));
    }
}
