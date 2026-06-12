use ab_glyph::{Font, FontArc, OutlineCurve, Point, ScaleFont};
use vectomancy::models::{BezierSegment, ColoredPath, Point2D};

#[allow(clippy::type_complexity)]
pub fn extract_text_outlines(
    text: &str,
    font_bytes: &[u8],
    size: f32,
    letter_spacing: f32,
) -> Result<(Vec<ColoredPath<Vec<BezierSegment>>>, (u32, u32)), String> {
    let font_vec = if woff2::decode::is_woff2(font_bytes) {
        let clean_bytes = if let Ok((_start, end)) = find_compressed_stream_bounds(font_bytes) {
            if end < font_bytes.len() {
                font_bytes[..end + 1].to_vec()
            } else {
                let mut cb = font_bytes[..end].to_vec();
                cb.push(0);
                cb
            }
        } else {
            font_bytes.to_vec()
        };
        let mut reader = &clean_bytes[..];
        woff2::decode::convert_woff2_to_ttf(&mut reader)
            .map_err(|e| format!("Failed to convert WOFF2 to TTF: {:?}", e))?
    } else {
        font_bytes.to_vec()
    };
    let font = FontArc::try_from_vec(font_vec).map_err(|e| e.to_string())?;
    let scaled = font.as_scaled(size);

    let units_per_em = font.units_per_em().unwrap_or(1000.0);
    let scale_factor = size / units_per_em;
    let ascent_scaled = scaled.ascent();

    let line_height = size * 1.2;
    let mut x_offset = 0.0;
    let mut max_x_offset = 0.0;
    let mut y_offset = 0.0;
    let mut paths = Vec::new();
    let mut prev_glyph_id = None;
    let processed_text = text.replace("\\n", "\n").replace("\\r", "\r");

    for c in processed_text.chars() {
        if c == '\n' || c == '\x0b' || c == '\x0c' || c == '\u{2028}' || c == '\u{2029}' {
            if x_offset > max_x_offset {
                max_x_offset = x_offset;
            }
            x_offset = 0.0;
            y_offset += line_height;
            prev_glyph_id = None;
            continue;
        }
        if c == '\r' {
            continue;
        }
        if c.is_control() {
            continue;
        }

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
        x_offset += h_advance + letter_spacing;
        prev_glyph_id = Some(glyph_id);

        if c == ' ' {
            continue;
        }

        if let Some(outline) = font.outline(glyph_id) {
            let scale_point = |p: Point| -> Point2D {
                Point2D {
                    x: ((p.x * scale_factor) + current_x) as f64,
                    y: (ascent_scaled - (p.y * scale_factor) + y_offset) as f64,
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
                    color_style: None,
                    data: segments,
                });
            }
        }
    }

    if x_offset > max_x_offset {
        max_x_offset = x_offset;
    }

    // Calculate default dimensions
    let width = if max_x_offset > 0.0 {
        max_x_offset.ceil() as u32
    } else {
        1
    };
    let height = if size > 0.0 {
        (y_offset + size * 1.5).ceil() as u32
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
    let (paths, _) = extract_text_outlines(&c.to_string(), font_bytes, size, 0.0)?;
    Ok(paths)
}

fn get_base_128(data: &[u8], offset: &mut usize) -> Result<u32, String> {
    let mut accum = 0u32;
    for i in 0..5 {
        if *offset >= data.len() {
            return Err("Truncated base 128".to_string());
        }
        let byte = data[*offset];
        *offset += 1;
        if i == 0 && byte == 0x80 {
            return Err("Leading zero in base 128".to_string());
        }
        if accum >> 25 != 0 {
            return Err("Overflow in base 128".to_string());
        }
        accum = (accum << 7) | ((byte & 0x7F) as u32);
        if byte & 0x80 == 0 {
            return Ok(accum);
        }
    }
    Err("More than 5 bytes in base 128".to_string())
}

fn get_255_u16(data: &[u8], offset: &mut usize) -> Result<u16, String> {
    if *offset >= data.len() {
        return Err("Truncated 255_u16".to_string());
    }
    let code = data[*offset];
    *offset += 1;
    match code {
        253 => {
            if *offset + 2 > data.len() {
                return Err("Truncated 255_u16 word".to_string());
            }
            let val = u16::from_be_bytes(data[*offset..*offset + 2].try_into().unwrap());
            *offset += 2;
            Ok(val)
        }
        255 => {
            if *offset >= data.len() {
                return Err("Truncated 255_u16 byte 1".to_string());
            }
            let val = data[*offset] as u16 + 253;
            *offset += 1;
            Ok(val)
        }
        254 => {
            if *offset >= data.len() {
                return Err("Truncated 255_u16 byte 2".to_string());
            }
            let val = data[*offset] as u16 + 506;
            *offset += 1;
            Ok(val)
        }
        _ => Ok(code as u16),
    }
}

pub(crate) fn find_compressed_stream_bounds(font_bytes: &[u8]) -> Result<(usize, usize), String> {
    if font_bytes.len() < 48 {
        return Err("Truncated header".to_string());
    }
    let signature = &font_bytes[0..4];
    if signature != b"wOF2" {
        return Err("Invalid magic word".to_string());
    }
    let flavor = &font_bytes[4..8];
    let num_tables = u16::from_be_bytes(
        font_bytes[12..14]
            .try_into()
            .map_err(|e: std::array::TryFromSliceError| e.to_string())?,
    );
    let total_compressed_size = u32::from_be_bytes(
        font_bytes[20..24]
            .try_into()
            .map_err(|e: std::array::TryFromSliceError| e.to_string())?,
    ) as usize;

    let mut offset = 48;

    for _ in 0..num_tables {
        if offset >= font_bytes.len() {
            return Err("Truncated table directory".to_string());
        }
        let flags = font_bytes[offset];
        offset += 1;
        let preprocessing_transformation_version = flags & 0xC0;
        let table_ref = flags & 0x3f;

        let mut tag = [0u8; 4];
        if table_ref == 0x3f {
            if offset + 4 > font_bytes.len() {
                return Err("Truncated table directory tag".to_string());
            }
            tag.copy_from_slice(&font_bytes[offset..offset + 4]);
            offset += 4;
        } else {
            if table_ref == 10 {
                tag = *b"glyf";
            } else if table_ref == 11 {
                tag = *b"loca";
            }
        }

        let _orig_length = get_base_128(font_bytes, &mut offset)?;
        let is_null_transform = if tag == *b"glyf" || tag == *b"loca" {
            preprocessing_transformation_version == 0xC0
        } else {
            preprocessing_transformation_version == 0x00
        };
        if !is_null_transform {
            let _transform_length = get_base_128(font_bytes, &mut offset)?;
        }
    }

    if flavor == b"ttcf" {
        if offset + 4 > font_bytes.len() {
            return Err("Truncated collection header version".to_string());
        }
        offset += 4;
        let num_fonts = get_255_u16(font_bytes, &mut offset)?;
        for _ in 0..num_fonts {
            let num_tables = get_255_u16(font_bytes, &mut offset)?;
            if offset + 4 > font_bytes.len() {
                return Err("Truncated font entry flavor".to_string());
            }
            offset += 4;
            for _ in 0..num_tables {
                let _index = get_255_u16(font_bytes, &mut offset)?;
            }
        }
    }

    Ok((offset, offset + total_compressed_size))
}
