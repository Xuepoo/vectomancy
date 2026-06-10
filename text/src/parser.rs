use ab_glyph::{Font, FontArc, OutlineCurve, Point, ScaleFont};
use vectomancy::models::{BezierSegment, ColoredPath, Point2D};

#[allow(clippy::type_complexity)]
pub fn extract_text_outlines(
    text: &str,
    font_bytes: &[u8],
    size: f32,
) -> Result<(Vec<ColoredPath<Vec<BezierSegment>>>, (u32, u32)), String> {
    let font = FontArc::try_from_vec(font_bytes.to_vec()).map_err(|e| e.to_string())?;
    let scaled = font.as_scaled(size);

    let units_per_em = font.units_per_em().unwrap_or(1000.0);
    let scale_factor = size / units_per_em;
    let ascent_scaled = scaled.ascent();

    let mut x_offset = 0.0;
    let mut paths = Vec::new();

    let mut prev_glyph_id = None;

    for c in text.chars() {
        let mut glyph_id = font.glyph_id(c);
        if glyph_id.0 == 0 && c != ' ' {
            // Glyph missing fallback: replace with tofu block ⬚
            let fallback_char = '⬚';
            glyph_id = font.glyph_id(fallback_char);
            if glyph_id.0 == 0 {
                // If ⬚ is also missing, fall back to ?
                glyph_id = font.glyph_id('?');
            }
        }

        if let Some(prev) = prev_glyph_id {
            x_offset += scaled.kern(prev, glyph_id);
        }

        let current_x = x_offset;
        let h_advance = scaled.h_advance(glyph_id);
        x_offset += h_advance;
        prev_glyph_id = Some(glyph_id);

        if c == ' ' {
            continue;
        }

        if let Some(outline) = font.outline(glyph_id) {
            let scale_point = |p: Point| -> Point2D {
                Point2D {
                    x: ((p.x * scale_factor) + current_x) as f64,
                    y: (ascent_scaled - (p.y * scale_factor)) as f64,
                }
            };

            let mut segments = Vec::new();
            let mut current_point: Option<Point2D> = None;

            for curve in outline.curves {
                let (p_start, p_end, segment_to_push) = match curve {
                    OutlineCurve::Line(p1, p2) => {
                        let sp1 = scale_point(p1);
                        let sp2 = scale_point(p2);
                        (sp1, sp2, BezierSegment::LineTo(sp2))
                    }
                    OutlineCurve::Quad(p1, p2, p3) => {
                        let sp1 = scale_point(p1);
                        let sp2 = scale_point(p2);
                        let sp3 = scale_point(p3);
                        (sp1, sp3, BezierSegment::QuadraticTo(sp2, sp3))
                    }
                    OutlineCurve::Cubic(p1, p2, p3, p4) => {
                        let sp1 = scale_point(p1);
                        let sp2 = scale_point(p2);
                        let sp3 = scale_point(p3);
                        let sp4 = scale_point(p4);
                        (sp1, sp4, BezierSegment::CubicTo(sp2, sp3, sp4))
                    }
                };

                let needs_move = match current_point {
                    None => true,
                    Some(cp) => {
                        let dist = ((cp.x - p_start.x).powi(2) + (cp.y - p_start.y).powi(2)).sqrt();
                        if dist > 1e-3 {
                            segments.push(BezierSegment::Close);
                            true
                        } else {
                            false
                        }
                    }
                };

                if needs_move {
                    segments.push(BezierSegment::MoveTo(p_start));
                }

                segments.push(segment_to_push);
                current_point = Some(p_end);
            }

            if current_point.is_some() {
                segments.push(BezierSegment::Close);
            }

            if !segments.is_empty() {
                paths.push(ColoredPath {
                    color_rgb: None,
                    data: segments,
                });
            }
        }
    }

    // Calculate default dimensions
    let width = if x_offset > 0.0 {
        x_offset.ceil() as u32
    } else {
        1
    };
    let height = if size > 0.0 {
        (size * 1.5).ceil() as u32
    } else {
        1
    };

    Ok((paths, (width, height)))
}

pub fn extract_char_outline(
    c: char,
    font_bytes: &[u8],
    size: f32,
) -> Result<Vec<ColoredPath<Vec<BezierSegment>>>, String> {
    let (paths, _) = extract_text_outlines(&c.to_string(), font_bytes, size)?;
    Ok(paths)
}
