use crate::error::VectomancyError;
use crate::models::{ColorStyle, ColoredPath, MathExpressionAST};
use std::fmt::Write as _;

/// Number of samples per Fourier stroke period, matching native raster tessellation density.
const FOURIER_STEPS: usize = 720;
/// Number of samples per spline segment when flattening to a polyline for the `<path>` `d` attribute.
const SPLINE_STEPS: usize = 64;

fn fmt_num(v: f64) -> String {
    // Trim to 4 decimal places; drop trailing zeros to keep markup compact.
    let rounded = (v * 10000.0).round() / 10000.0;
    let mut s = format!("{:.4}", rounded);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
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
) -> String {
    match style {
        None => "#000000".to_string(),
        Some(ColorStyle::Solid(rgb)) => {
            let r = (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8;
            format!("rgb({}, {}, {})", r, g, b)
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
                x0 = fmt_num(x0),
                y0 = fmt_num(y0),
                x1 = fmt_num(x1),
                y1 = fmt_num(y1),
            );
            for (offset, rgb) in &grad.stops {
                let r = (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8;
                let g = (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8;
                let b = (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8;
                let _ = write!(
                    defs,
                    r#"<stop offset="{}" stop-color="rgb({}, {}, {})"/>"#,
                    fmt_num(offset.clamp(0.0, 1.0) as f64),
                    r,
                    g,
                    b
                );
            }
            defs.push_str("</linearGradient>");

            format!("url(#{})", id)
        }
    }
}

fn polyline_path(points: impl Iterator<Item = (f64, f64)>) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.enumerate() {
        if i == 0 {
            let _ = write!(d, "M {} {}", fmt_num(x), fmt_num(y));
        } else {
            let _ = write!(d, " L {} {}", fmt_num(x), fmt_num(y));
        }
    }
    d
}

fn emit_polyline_paths(
    paths: &[ColoredPath<Vec<crate::models::Point2D>>],
    bbox: [f32; 4],
    defs: &mut String,
    next_grad_id: &mut usize,
    body: &mut String,
    stroke_width: f32,
) {
    for path in paths {
        if path.data.is_empty() {
            continue;
        }
        let stroke = resolve_stroke(&path.color_style, bbox, defs, next_grad_id);
        let d = polyline_path(path.data.iter().map(|p| (p.x, p.y)));
        let _ = write!(
            body,
            r#"<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{}"/>"#,
            stroke_width
        );
    }
}

fn emit_spline_paths(
    equations: &[ColoredPath<Vec<crate::models::SplineEquation>>],
    bbox: [f32; 4],
    defs: &mut String,
    next_grad_id: &mut usize,
    body: &mut String,
    stroke_width: f32,
) {
    for path in equations {
        if path.data.is_empty() {
            continue;
        }
        let stroke = resolve_stroke(&path.color_style, bbox, defs, next_grad_id);
        let mut points = Vec::new();
        for eq in &path.data {
            for i in 0..=SPLINE_STEPS {
                let t = i as f64 / SPLINE_STEPS as f64;
                let mut x = 0.0;
                let mut y = 0.0;
                for (j, coef) in eq.x_poly.iter().enumerate() {
                    x += coef * t.powi(j as i32);
                }
                for (j, coef) in eq.y_poly.iter().enumerate() {
                    y += coef * t.powi(j as i32);
                }
                points.push((x, y));
            }
        }
        let d = polyline_path(points.into_iter());
        let _ = write!(
            body,
            r#"<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{}"/>"#,
            stroke_width
        );
    }
}

fn emit_fourier_paths(
    strokes: &[ColoredPath<Vec<crate::models::FourierTerm>>],
    bbox: [f32; 4],
    defs: &mut String,
    next_grad_id: &mut usize,
    body: &mut String,
    stroke_width: f32,
) {
    for path in strokes {
        if path.data.is_empty() {
            continue;
        }
        let stroke = resolve_stroke(&path.color_style, bbox, defs, next_grad_id);
        let mut points = Vec::with_capacity(FOURIER_STEPS + 1);
        for i in 0..=FOURIER_STEPS {
            let t = i as f64 / FOURIER_STEPS as f64 * std::f64::consts::TAU;
            let mut x = 0.0;
            let mut y = 0.0;
            for term in &path.data {
                let angle = term.frequency * t + term.phase;
                x += term.amplitude * angle.cos();
                y += term.amplitude * angle.sin();
            }
            points.push((x, y));
        }
        let d = polyline_path(points.into_iter());
        let _ = write!(
            body,
            r#"<path d="{d}" fill="none" stroke="{stroke}" stroke-width="{}"/>"#,
            stroke_width
        );
    }
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
            );
        }
        MathExpressionAST::Spline { equations, .. } => {
            emit_spline_paths(
                equations,
                bbox,
                &mut defs,
                &mut next_grad_id,
                &mut body,
                stroke_width,
            );
        }
        MathExpressionAST::Polyline { paths, .. } => {
            emit_polyline_paths(
                paths,
                bbox,
                &mut defs,
                &mut next_grad_id,
                &mut body,
                stroke_width,
            );
        }
    }

    let (width, height) = original_dimensions;
    let mut out = String::new();
    let _ = write!(
        out,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
"#,
        width = width,
        height = height,
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
