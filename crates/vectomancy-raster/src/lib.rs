use image::GenericImageView;
use rayon::prelude::*;
use std::path::Path;
use tracing::info;
use vectomancy_geometry::{Point2D, Polyline, StyledPath};

fn sobel_gradients_parallel(image: &image::GrayImage) -> Vec<u16> {
    use imageproc::kernel::{SOBEL_HORIZONTAL_3X3, SOBEL_VERTICAL_3X3};

    let (horizontal, vertical) = rayon::join(
        || imageproc::filter::filter_clamped_parallel::<_, _, i16>(image, SOBEL_HORIZONTAL_3X3),
        || imageproc::filter::filter_clamped_parallel::<_, _, i16>(image, SOBEL_VERTICAL_3X3),
    );

    let (width, height) = image.dimensions();
    let mut gradients = vec![0u16; (width * height) as usize];

    gradients
        .par_chunks_exact_mut(width as usize)
        .enumerate()
        .for_each(|(y, row)| {
            let y_u32 = y as u32;
            for x in 0..width {
                let g_x = horizontal.get_pixel(x, y_u32)[0] as f32;
                let g_y = vertical.get_pixel(x, y_u32)[0] as f32;
                let g = (g_x * g_x + g_y * g_y).sqrt();
                row[x as usize] = g.clamp(0.0, u16::MAX as f32) as u16;
            }
        });

    gradients
}

#[allow(clippy::type_complexity)]
pub fn decode_raster_memory(
    bytes: &[u8],
    color: bool,
) -> Result<(Vec<StyledPath<Polyline>>, (u32, u32)), String> {
    info!("Processing raster image from memory in vectomancy-raster");
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let dimensions = (img.width(), img.height());

    let gray = img.to_luma8();
    let gradients = sobel_gradients_parallel(&gray);

    let threshold = imageproc::contrast::otsu_level(&gray);
    let (width, height) = dimensions;

    let mut binary = vec![false; (width * height) as usize];

    binary
        .par_chunks_exact_mut(width as usize)
        .enumerate()
        .for_each(|(y, row)| {
            let y_u32 = y as u32;
            for x in 0..width {
                let idx = (y_u32 * width + x) as usize;
                if gradients[idx] > (threshold as u16) {
                    row[x as usize] = true;
                }
            }
        });

    // Skeletonize & extract polylines
    let mut polylines = Vec::new();
    let mut visited = vec![false; (width * height) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if binary[idx] && !visited[idx] {
                let mut pts = Vec::new();
                let mut curr_x = x;
                let mut curr_y = y;

                while curr_x < width && curr_y < height {
                    let c_idx = (curr_y * width + curr_x) as usize;
                    if !binary[c_idx] || visited[c_idx] {
                        break;
                    }
                    visited[c_idx] = true;
                    pts.push(Point2D {
                        x: curr_x as f64,
                        y: curr_y as f64,
                    });

                    // Search neighbors
                    let mut found = false;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = curr_x as i32 + dx;
                            let ny = curr_y as i32 + dy;
                            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                                let n_idx = (ny as u32 * width + nx as u32) as usize;
                                if binary[n_idx] && !visited[n_idx] {
                                    curr_x = nx as u32;
                                    curr_y = ny as u32;
                                    found = true;
                                    break;
                                }
                            }
                        }
                        if found {
                            break;
                        }
                    }
                    if !found {
                        break;
                    }
                }

                if pts.len() >= 2 {
                    let style = if color {
                        let pixel = img.get_pixel(x, y);
                        Some(format!("#{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2]))
                    } else {
                        None
                    };

                    polylines.push(StyledPath {
                        geometry: Polyline {
                            points: pts,
                            closed: false,
                        },
                        color_style: style,
                    });
                }
            }
        }
    }

    Ok((polylines, dimensions))
}

#[allow(clippy::type_complexity)]
pub fn decode_raster_file(
    path: &Path,
    color: bool,
) -> Result<(Vec<StyledPath<Polyline>>, (u32, u32)), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    decode_raster_memory(&bytes, color)
}
