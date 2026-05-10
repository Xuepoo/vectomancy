use crate::cli::OutputFormat;
use crate::error::VectomancyError;
use crate::models::MathExpressionAST;
use std::path::Path;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

#[allow(clippy::too_many_arguments)]
pub fn render_to_image(
    ast: &MathExpressionAST,
    output_path: &Path,
    format: &OutputFormat,
    transparent: bool,
    original_dimensions: (u32, u32),
    target_dimensions: (u32, u32),
    stroke_width: f32,
    bit_depth: Option<u8>,
    color_space: Option<String>,
) -> Result<(), VectomancyError> {
    if let Some(cs) = color_space {
        tracing::debug!(
            "Color space {} requested, but ICC profiles are not fully supported yet.",
            cs
        );
    }
    let mut pixmap = Pixmap::new(target_dimensions.0, target_dimensions.1).ok_or_else(|| {
        VectomancyError::MemoryAllocationFailed("Failed to allocate pixmap".to_string())
    })?;

    if !transparent {
        pixmap.fill(Color::WHITE);
    }

    let scale_x = target_dimensions.0 as f32 / original_dimensions.0 as f32;
    let scale_y = target_dimensions.1 as f32 / original_dimensions.1 as f32;
    let scale = scale_x.min(scale_y);

    let offset_x = (target_dimensions.0 as f32 - original_dimensions.0 as f32 * scale) / 2.0;
    let offset_y = (target_dimensions.1 as f32 - original_dimensions.1 as f32 * scale) / 2.0;

    let transform = Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);

    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };

    let stroke = Stroke {
        width: stroke_width,
        ..Default::default()
    };

    match ast {
        MathExpressionAST::Polyline { paths } => {
            for path in paths {
                let mut pb = PathBuilder::new();
                let mut first = true;
                for pt in &path.data {
                    if first {
                        pb.move_to(pt.x as f32, pt.y as f32);
                        first = false;
                    } else {
                        pb.line_to(pt.x as f32, pt.y as f32);
                    }
                }
                if let Some(skia_path) = pb.finish() {
                    if let Some((r, g, b)) = path.color_rgb {
                        paint.set_color_rgba8(r, g, b, 255);
                    } else {
                        paint.set_color_rgba8(0, 0, 0, 255);
                    }
                    pixmap.stroke_path(&skia_path, &paint, &stroke, transform, None);
                }
            }
        }
        MathExpressionAST::Spline { equations } => {
            for path in equations {
                let mut pb = PathBuilder::new();
                let mut first = true;
                for eq in &path.data {
                    let steps = 50;
                    for i in 0..=steps {
                        let t = i as f64 / steps as f64;
                        let mut x = 0.0;
                        let mut y = 0.0;
                        for (j, coef) in eq.x_poly.iter().enumerate() {
                            x += coef * t.powi(j as i32);
                        }
                        for (j, coef) in eq.y_poly.iter().enumerate() {
                            y += coef * t.powi(j as i32);
                        }
                        if first {
                            pb.move_to(x as f32, y as f32);
                            first = false;
                        } else {
                            pb.line_to(x as f32, y as f32);
                        }
                    }
                }
                if let Some(skia_path) = pb.finish() {
                    if let Some((r, g, b)) = path.color_rgb {
                        paint.set_color_rgba8(r, g, b, 255);
                    } else {
                        paint.set_color_rgba8(0, 0, 0, 255);
                    }
                    pixmap.stroke_path(&skia_path, &paint, &stroke, transform, None);
                }
            }
        }
        MathExpressionAST::Fourier { strokes } => {
            let steps = target_dimensions.0.max(target_dimensions.1) as usize;
            for path in strokes {
                let mut pb = PathBuilder::new();
                let mut first = true;
                for i in 0..=steps {
                    let t = i as f64 / steps as f64;
                    let mut x = 0.0;
                    let mut y = 0.0;
                    for term in &path.data {
                        let angle = term.frequency * t * std::f64::consts::TAU + term.phase;
                        x += term.amplitude * angle.cos();
                        y += term.amplitude * angle.sin();
                    }
                    if first {
                        pb.move_to(x as f32, y as f32);
                        first = false;
                    } else {
                        pb.line_to(x as f32, y as f32);
                    }
                }
                if let Some(skia_path) = pb.finish() {
                    if let Some((r, g, b)) = path.color_rgb {
                        paint.set_color_rgba8(r, g, b, 255);
                    } else {
                        paint.set_color_rgba8(0, 0, 0, 255);
                    }
                    pixmap.stroke_path(&skia_path, &paint, &stroke, transform, None);
                }
            }
        }
    }

    let img_data = pixmap
        .encode_png()
        .map_err(|e| VectomancyError::InvalidInput(format!("PNG encoding error: {}", e)))?;
    let mut img = image::load_from_memory(&img_data)
        .map_err(|e| VectomancyError::InvalidInput(format!("Image loading error: {}", e)))?;

    if bit_depth == Some(16) {
        if transparent {
            let rgba16 = img.into_rgba16();
            img = image::DynamicImage::ImageRgba16(rgba16);
        } else {
            let rgb16 = img.into_rgb16();
            img = image::DynamicImage::ImageRgb16(rgb16);
        }
    }

    match format {
        OutputFormat::Png => {
            img.save_with_format(output_path, image::ImageFormat::Png)
                .map_err(|e| VectomancyError::InvalidInput(format!("Image save error: {}", e)))?;
        }
        OutputFormat::Jpg => {
            img.into_rgb8()
                .save_with_format(output_path, image::ImageFormat::Jpeg)
                .map_err(|e| VectomancyError::InvalidInput(format!("Image save error: {}", e)))?;
        }
        OutputFormat::Webp => {
            img.save_with_format(output_path, image::ImageFormat::WebP)
                .map_err(|e| VectomancyError::InvalidInput(format!("Image save error: {}", e)))?;
        }
        _ => {
            return Err(VectomancyError::InvalidInput(
                "Unsupported format for native rendering".to_string(),
            ))
        }
    }

    Ok(())
}
