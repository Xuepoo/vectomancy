use crate::error::VectomancyError;
use crate::models::Point2D;
use image::Luma;
use std::path::Path;
use tracing::{debug, info};

pub fn process_raster_image(path: &Path) -> Result<Vec<Vec<Point2D>>, VectomancyError> {
    info!("Processing raster image: {:?}", path);
    let img = image::open(path).map_err(|e| VectomancyError::ImageProcessing(e.to_string()))?;

    // 1. Grayscale
    debug!("Converting to grayscale");
    let grayscale = img.into_luma8();

    debug!("Applying Sobel edge detection");
    let (width, height) = grayscale.dimensions();
    let mut edge_image = image::GrayImage::new(width, height);
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let p00 = grayscale.get_pixel(x - 1, y - 1).0[0] as f32;
            let p10 = grayscale.get_pixel(x, y - 1).0[0] as f32;
            let p20 = grayscale.get_pixel(x + 1, y - 1).0[0] as f32;
            let p01 = grayscale.get_pixel(x - 1, y).0[0] as f32;
            let p21 = grayscale.get_pixel(x + 1, y).0[0] as f32;
            let p02 = grayscale.get_pixel(x - 1, y + 1).0[0] as f32;
            let p12 = grayscale.get_pixel(x, y + 1).0[0] as f32;
            let p22 = grayscale.get_pixel(x + 1, y + 1).0[0] as f32;

            let gx = -p00 + p20 - 2.0 * p01 + 2.0 * p21 - p02 + p22;
            let gy = -p00 - 2.0 * p10 - p20 + p02 + 2.0 * p12 + p22;

            let mag = (gx * gx + gy * gy).sqrt();
            let val = if mag > 255.0 { 255 } else { mag as u8 };
            edge_image.put_pixel(x, y, Luma([val]));
        }
    }

    // 2. Otsu Binarization
    debug!("Applying Otsu binarization");

    let mut histogram = [0u32; 256];
    for pixel in edge_image.pixels() {
        histogram[pixel.0[0] as usize] += 1;
    }

    let total_pixels = width * height;
    let mut sum = 0.0;
    for (i, &count) in histogram.iter().enumerate() {
        sum += i as f64 * count as f64;
    }

    let mut sum_b = 0.0;
    let mut w_b = 0;
    let mut w_f;

    let mut var_max = 0.0;
    let mut threshold = 0u8;

    for (i, &count) in histogram.iter().enumerate() {
        w_b += count;
        if w_b == 0 {
            continue;
        }

        w_f = total_pixels - w_b;
        if w_f == 0 {
            break;
        }

        sum_b += i as f64 * count as f64;

        let m_b = sum_b / w_b as f64;
        let m_f = (sum - sum_b) / w_f as f64;

        let var_between = w_b as f64 * w_f as f64 * (m_b - m_f).powi(2);

        if var_between > var_max {
            var_max = var_between;
            threshold = i as u8;
        }
    }

    info!("Otsu calculated threshold: {}", threshold);

    let padded_width = width as usize + 2;
    let padded_height = height as usize + 2;
    let mut grid = vec![vec![false; padded_width]; padded_height];

    for (x, y, pixel) in edge_image.enumerate_pixels() {
        let Luma([luma]) = *pixel;
        if luma > threshold {
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

    Ok(all_paths)
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
