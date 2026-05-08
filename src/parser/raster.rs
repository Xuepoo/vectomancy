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

    // 2. Otsu Binarization
    debug!("Applying Otsu binarization");
    let (width, height) = grayscale.dimensions();

    let mut histogram = [0u32; 256];
    for pixel in grayscale.pixels() {
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

    // Create a padded grid for thinning (true = foreground/black, false = background/white)
    let padded_width = width as usize + 2;
    let padded_height = height as usize + 2;
    let mut grid = vec![vec![false; padded_width]; padded_height];

    for (x, y, pixel) in grayscale.enumerate_pixels() {
        let Luma([luma]) = *pixel;
        if luma <= threshold {
            grid[y as usize + 1][x as usize + 1] = true;
        }
    }

    // 3. Moore Neighborhood Tracing (Boundary Tracing)
    debug!("Extracting contours using Moore boundary tracing");
    let all_paths = trace_contours(&grid, padded_width, padded_height);

    let total_pts: usize = all_paths.iter().map(|p| p.len()).sum();
    info!(
        "Extracted {} skeleton paths (total {} points) from image",
        all_paths.len(),
        total_pts
    );

    Ok(all_paths)
}

fn trace_contours(grid: &[Vec<bool>], width: usize, height: usize) -> Vec<Vec<Point2D>> {
    let mut all_paths = Vec::new();
    let mut visited = vec![vec![false; width]; height];

    // Scan the grid for unvisited foreground pixels
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if !visited[y][x] && grid[y][x] {
                // Found an unvisited foreground pixel; trace its contour
                if let Some(contour) = trace_single_contour(grid, x, y, &mut visited, width, height)
                {
                    if contour.len() > 3 {
                        all_paths.push(contour);
                    }
                }
            }
        }
    }

    all_paths
}

fn trace_single_contour(
    grid: &[Vec<bool>],
    start_x: usize,
    start_y: usize,
    visited: &mut [Vec<bool>],
    width: usize,
    height: usize,
) -> Option<Vec<Point2D>> {
    let mut contour = vec![Point2D {
        x: (start_x as f64) - 1.0,
        y: (start_y as f64) - 1.0,
    }];

    let mut x = start_x;
    let mut y = start_y;
    visited[y][x] = true;

    // Directions: 8 neighbors in clockwise order (East, SE, South, SW, West, NW, North, NE)
    let directions = [
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ];
    let mut dir_idx = 0;

    loop {
        let mut found_next = false;

        for _ in 0..8 {
            let (dx, dy) = directions[dir_idx];
            let next_x = (x as i32 + dx) as usize;
            let next_y = (y as i32 + dy) as usize;

            if next_x > 0
                && next_x < width - 1
                && next_y > 0
                && next_y < height - 1
                && grid[next_y][next_x]
            {
                // Found a neighboring foreground pixel
                if (next_x, next_y) == (start_x, start_y) && contour.len() > 4 {
                    // We've returned to start and traced enough points
                    return Some(contour);
                }

                if !visited[next_y][next_x] {
                    x = next_x;
                    y = next_y;
                    visited[y][x] = true;
                    contour.push(Point2D {
                        x: (x as f64) - 1.0,
                        y: (y as f64) - 1.0,
                    });
                    dir_idx = (dir_idx + 6) % 8; // Adjust direction for next iteration
                    found_next = true;
                    break;
                }
            }

            dir_idx = (dir_idx + 1) % 8;
        }

        if !found_next {
            // Dead end or completed contour
            if contour.len() > 2 {
                return Some(contour);
            }
            return None;
        }

        // Prevent infinite loops on very small contours
        if contour.len() > 100000 {
            return Some(contour);
        }
    }
}
