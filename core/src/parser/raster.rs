use crate::error::VectomancyError;
use crate::models::{ColoredPath, Point2D};
use image::Luma;
use std::path::Path;
use tracing::{debug, info};

#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

#[allow(clippy::type_complexity)]
pub fn process_raster_image(
    path: &Path,
    color: bool,
) -> Result<(Vec<ColoredPath<Vec<Point2D>>>, (u32, u32)), VectomancyError> {
    info!("Processing raster image: {:?}", path);
    let img = image::open(path).map_err(|e| VectomancyError::ImageProcessing(e.to_string()))?;
    process_raster_image_core(img, color)
}

#[allow(clippy::type_complexity)]
pub fn process_raster_from_memory(
    bytes: &[u8],
    color: bool,
) -> Result<(Vec<ColoredPath<Vec<Point2D>>>, (u32, u32)), VectomancyError> {
    info!("Processing raster image from memory");
    let img = image::load_from_memory(bytes)
        .map_err(|e| VectomancyError::ImageProcessing(e.to_string()))?;
    process_raster_image_core(img, color)
}

#[allow(clippy::type_complexity)]
fn process_raster_image_core(
    img: image::DynamicImage,
    color: bool,
) -> Result<(Vec<ColoredPath<Vec<Point2D>>>, (u32, u32)), VectomancyError> {
    // 1. Grayscale
    debug!("Converting to grayscale");
    let grayscale = img.to_luma8();

    debug!("Applying Sobel edge detection");
    let (width, height) = grayscale.dimensions();
    info!(
        "Image loaded successfully. Dimensions: {}x{}, Color Mode: {}",
        width,
        height,
        if color { "RGB" } else { "Grayscale" }
    );
    let sobel16 = imageproc::gradients::sobel_gradients(&grayscale);
    let mut edge_image = image::GrayImage::new(width, height);
    for (x, y, pixel) in sobel16.enumerate_pixels() {
        let val = if pixel.0[0] > 255 {
            255
        } else {
            pixel.0[0] as u8
        };
        edge_image.put_pixel(x, y, Luma([val]));
    }

    // 2. Otsu Binarization
    debug!("Applying Otsu binarization");
    let threshold_val = imageproc::contrast::otsu_level(&edge_image);
    info!("Otsu calculated threshold: {}", threshold_val);

    let padded_width = width as usize + 2;
    let padded_height = height as usize + 2;
    let mut grid = vec![vec![false; padded_width]; padded_height];

    for (x, y, pixel) in edge_image.enumerate_pixels() {
        let Luma([luma]) = *pixel;
        if luma > threshold_val {
            grid[y as usize + 1][x as usize + 1] = true;
        }
    }

    // 3. Zhang-Suen Thinning
    debug!("Applying Zhang-Suen thinning");
    zhang_suen_thinning(&mut grid, padded_width, padded_height);

    // 4. Extract paths using graph traversal
    debug!("Extracting paths from thinned skeleton");
    let all_paths = extract_paths(&grid, padded_width, padded_height);

    let total_pts: usize = all_paths.iter().map(|p| p.len()).sum();
    info!(
        "Extracted {} skeleton paths (total {} points) from image",
        all_paths.len(),
        total_pts
    );

    let rgb_image = if color { Some(img.to_rgb8()) } else { None };

    #[cfg(not(target_arch = "wasm32"))]
    let path_iter = all_paths.into_par_iter();
    #[cfg(target_arch = "wasm32")]
    let path_iter = all_paths.into_iter();

    let colored_paths: Vec<_> = path_iter
        .map(|path| {
            let color_rgb = if let Some(ref rgb) = rgb_image {
                let mut r_sum = 0u64;
                let mut g_sum = 0u64;
                let mut b_sum = 0u64;
                let mut count = 0u64;
                for pt in &path {
                    let x = pt.x.round() as u32;
                    let y = pt.y.round() as u32;
                    if x < width && y < height {
                        let pixel = rgb.get_pixel(x, y);
                        r_sum += pixel[0] as u64;
                        g_sum += pixel[1] as u64;
                        b_sum += pixel[2] as u64;
                        count += 1;
                    }
                }
                #[allow(clippy::manual_checked_ops)]
                if count > 0 {
                    Some((
                        (r_sum / count) as u8,
                        (g_sum / count) as u8,
                        (b_sum / count) as u8,
                    ))
                } else {
                    None
                }
            } else {
                None
            };
            ColoredPath {
                color_rgb,
                data: path,
            }
        })
        .collect();

    Ok((colored_paths, (width, height)))
}

fn zhang_suen_thinning(grid: &mut [Vec<bool>], width: usize, height: usize) {
    let mut changed = true;
    while changed {
        changed = false;
        let mut to_delete = Vec::new();

        // Step 1
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                if !grid[y][x] {
                    continue;
                }
                let p2 = grid[y - 1][x] as u8;
                let p3 = grid[y - 1][x + 1] as u8;
                let p4 = grid[y][x + 1] as u8;
                let p5 = grid[y + 1][x + 1] as u8;
                let p6 = grid[y + 1][x] as u8;
                let p7 = grid[y + 1][x - 1] as u8;
                let p8 = grid[y][x - 1] as u8;
                let p9 = grid[y - 1][x - 1] as u8;

                let b = p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9;
                if !(2..=6).contains(&b) {
                    continue;
                }

                let mut a = 0;
                if p2 == 0 && p3 == 1 {
                    a += 1;
                }
                if p3 == 0 && p4 == 1 {
                    a += 1;
                }
                if p4 == 0 && p5 == 1 {
                    a += 1;
                }
                if p5 == 0 && p6 == 1 {
                    a += 1;
                }
                if p6 == 0 && p7 == 1 {
                    a += 1;
                }
                if p7 == 0 && p8 == 1 {
                    a += 1;
                }
                if p8 == 0 && p9 == 1 {
                    a += 1;
                }
                if p9 == 0 && p2 == 1 {
                    a += 1;
                }

                if a != 1 {
                    continue;
                }

                if p2 * p4 * p6 != 0 {
                    continue;
                }
                if p4 * p6 * p8 != 0 {
                    continue;
                }

                to_delete.push((x, y));
            }
        }

        if !to_delete.is_empty() {
            changed = true;
            for &(x, y) in &to_delete {
                grid[y][x] = false;
            }
            to_delete.clear();
        }

        // Step 2
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                if !grid[y][x] {
                    continue;
                }
                let p2 = grid[y - 1][x] as u8;
                let p3 = grid[y - 1][x + 1] as u8;
                let p4 = grid[y][x + 1] as u8;
                let p5 = grid[y + 1][x + 1] as u8;
                let p6 = grid[y + 1][x] as u8;
                let p7 = grid[y + 1][x - 1] as u8;
                let p8 = grid[y][x - 1] as u8;
                let p9 = grid[y - 1][x - 1] as u8;

                let b = p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9;
                if !(2..=6).contains(&b) {
                    continue;
                }

                let mut a = 0;
                if p2 == 0 && p3 == 1 {
                    a += 1;
                }
                if p3 == 0 && p4 == 1 {
                    a += 1;
                }
                if p4 == 0 && p5 == 1 {
                    a += 1;
                }
                if p5 == 0 && p6 == 1 {
                    a += 1;
                }
                if p6 == 0 && p7 == 1 {
                    a += 1;
                }
                if p7 == 0 && p8 == 1 {
                    a += 1;
                }
                if p8 == 0 && p9 == 1 {
                    a += 1;
                }
                if p9 == 0 && p2 == 1 {
                    a += 1;
                }

                if a != 1 {
                    continue;
                }

                if p2 * p4 * p8 != 0 {
                    continue;
                }
                if p2 * p6 * p8 != 0 {
                    continue;
                }

                to_delete.push((x, y));
            }
        }

        if !to_delete.is_empty() {
            changed = true;
            for &(x, y) in &to_delete {
                grid[y][x] = false;
            }
        }
    }
}

fn extract_paths(grid: &[Vec<bool>], width: usize, height: usize) -> Vec<Vec<Point2D>> {
    let mut paths = Vec::new();
    let mut visited = vec![vec![false; width]; height];

    let get_neighbors = |x: usize, y: usize| -> Vec<(usize, usize)> {
        let mut n = Vec::new();
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if grid[ny][nx] {
                        n.push((nx, ny));
                    }
                }
            }
        }
        n
    };

    // Find endpoints (pixels with exactly 1 neighbor)
    let mut endpoints = Vec::new();
    #[allow(clippy::needless_range_loop)]
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if grid[y][x] && get_neighbors(x, y).len() == 1 {
                endpoints.push((x, y));
            }
        }
    }

    // Trace from endpoints
    for &(start_x, start_y) in &endpoints {
        if visited[start_y][start_x] {
            continue;
        }

        let mut path = Vec::new();
        let mut curr_x = start_x;
        let mut curr_y = start_y;

        loop {
            visited[curr_y][curr_x] = true;
            path.push(Point2D {
                x: (curr_x as f64) - 1.0,
                y: (curr_y as f64) - 1.0,
            });

            let neighbors = get_neighbors(curr_x, curr_y);
            let mut next = None;

            for &(nx, ny) in &neighbors {
                if !visited[ny][nx] {
                    next = Some((nx, ny));
                    break;
                }
            }

            if let Some((nx, ny)) = next {
                curr_x = nx;
                curr_y = ny;
            } else {
                if path.len() >= 2 {
                    let prev_x = (path[path.len() - 2].x + 1.0) as usize;
                    let prev_y = (path[path.len() - 2].y + 1.0) as usize;

                    if let Some(&(nx, ny)) = neighbors
                        .iter()
                        .find(|&&(nx, ny)| nx != prev_x || ny != prev_y)
                    {
                        path.push(Point2D {
                            x: (nx as f64) - 1.0,
                            y: (ny as f64) - 1.0,
                        });
                    }
                } else if path.len() == 1 {
                    if let Some(&(nx, ny)) = neighbors.first() {
                        path.push(Point2D {
                            x: (nx as f64) - 1.0,
                            y: (ny as f64) - 1.0,
                        });
                    }
                }
                break;
            }
        }

        if path.len() > 1 {
            paths.push(path);
        }
    }

    // Trace remaining loops or isolated components
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if grid[y][x] && !visited[y][x] {
                let mut path = Vec::new();
                let mut curr_x = x;
                let mut curr_y = y;

                loop {
                    visited[curr_y][curr_x] = true;
                    path.push(Point2D {
                        x: (curr_x as f64) - 1.0,
                        y: (curr_y as f64) - 1.0,
                    });

                    let neighbors = get_neighbors(curr_x, curr_y);
                    let mut next = None;

                    for &(nx, ny) in &neighbors {
                        if !visited[ny][nx] {
                            next = Some((nx, ny));
                            break;
                        }
                    }

                    if let Some((nx, ny)) = next {
                        curr_x = nx;
                        curr_y = ny;
                    } else {
                        if path.len() >= 2 {
                            let prev_x = (path[path.len() - 2].x + 1.0) as usize;
                            let prev_y = (path[path.len() - 2].y + 1.0) as usize;

                            if let Some(&(nx, ny)) = neighbors
                                .iter()
                                .find(|&&(nx, ny)| nx != prev_x || ny != prev_y)
                            {
                                path.push(Point2D {
                                    x: (nx as f64) - 1.0,
                                    y: (ny as f64) - 1.0,
                                });
                            }
                        } else if path.len() == 1 {
                            if let Some(&(nx, ny)) = neighbors.first() {
                                path.push(Point2D {
                                    x: (nx as f64) - 1.0,
                                    y: (ny as f64) - 1.0,
                                });
                            }
                        }
                        break;
                    }
                }

                if path.len() > 1 {
                    paths.push(path);
                }
            }
        }
    }

    paths
}
