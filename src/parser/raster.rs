use crate::error::VectomancyError;
use crate::models::Point2D;
use image::{ImageBuffer, Luma};
use std::path::Path;
use tracing::{debug, info};

const NEIGHBORS: [(i32, i32); 8] = [
    (-1, 0),  // Left
    (-1, -1), // Top-Left
    (0, -1),  // Top
    (1, -1),  // Top-Right
    (1, 0),   // Right
    (1, 1),   // Bottom-Right
    (0, 1),   // Bottom
    (-1, 1),  // Bottom-Left
];

pub fn process_raster_image(path: &Path) -> Result<Vec<Point2D>, VectomancyError> {
    info!("Processing raster image: {:?}", path);
    let img = image::open(path).map_err(|e| VectomancyError::ImageProcessing(e.to_string()))?;

    // 1. Grayscale
    debug!("Converting to grayscale");
    let grayscale = img.into_luma8();

    // 2. Otsu Binarization (simplified thresholding for now)
    debug!("Applying binarization");
    let threshold = 128u8; // A simple threshold. In a real app, calculate Otsu's.
    let (width, height) = grayscale.dimensions();
    let mut binarized = ImageBuffer::new(width, height);
    for (x, y, pixel) in grayscale.enumerate_pixels() {
        let Luma([luma]) = *pixel;
        let new_pixel = if luma > threshold {
            Luma([255u8])
        } else {
            Luma([0u8])
        };
        binarized.put_pixel(x, y, new_pixel);
    }

    // 3. Extract points using Moore-Neighbor contour tracing
    debug!("Extracting points using Moore-Neighbor tracing");
    let mut all_boundaries = Vec::new();
    let mut visited = vec![false; (width * height) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if !visited[idx] && binarized.get_pixel(x, y).0[0] == 0 {
                // Found a new component!
                // 1. Trace its boundary
                let boundary = trace_boundary(x, y, &binarized, width, height);
                all_boundaries.extend(boundary);

                // 2. Flood fill to mark the whole component as visited
                flood_fill(x, y, &binarized, &mut visited, width, height);
            }
        }
    }

    info!(
        "Extracted {} boundary points from image",
        all_boundaries.len()
    );
    Ok(all_boundaries)
}

fn trace_boundary(
    start_x: u32,
    start_y: u32,
    img: &ImageBuffer<Luma<u8>, Vec<u8>>,
    width: u32,
    height: u32,
) -> Vec<Point2D> {
    let mut boundary = Vec::new();
    let start_p = (start_x as i32, start_y as i32);
    let mut b = start_p;
    let mut c = (start_p.0 - 1, start_p.1); // The pixel to the left is guaranteed to be white or out of bounds
    let c_start = c;

    boundary.push(Point2D {
        x: b.0 as f64,
        y: b.1 as f64,
    });

    loop {
        let dx = c.0 - b.0;
        let dy = c.1 - b.1;

        let start_idx = NEIGHBORS.iter().position(|&n| n == (dx, dy)).unwrap_or(0);

        let mut found = false;
        let mut next_b = b;
        let mut next_c = c;

        for i in 1..=8 {
            let idx = (start_idx + i) % 8;
            let nx = b.0 + NEIGHBORS[idx].0;
            let ny = b.1 + NEIGHBORS[idx].1;

            let is_black = if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                img.get_pixel(nx as u32, ny as u32).0[0] == 0
            } else {
                false
            };

            if is_black {
                next_b = (nx, ny);
                // next_c is the white pixel we examined just before next_b
                let prev_idx = (start_idx + i - 1) % 8;
                next_c = (b.0 + NEIGHBORS[prev_idx].0, b.1 + NEIGHBORS[prev_idx].1);
                found = true;
                break;
            }
        }

        if !found {
            // Isolated pixel
            break;
        }

        // Jacob's stopping criterion
        if next_b == start_p && next_c == c_start {
            break;
        }

        b = next_b;
        c = next_c;
        boundary.push(Point2D {
            x: b.0 as f64,
            y: b.1 as f64,
        });
    }

    boundary
}

fn flood_fill(
    start_x: u32,
    start_y: u32,
    img: &ImageBuffer<Luma<u8>, Vec<u8>>,
    visited: &mut [bool],
    width: u32,
    height: u32,
) {
    let mut stack = vec![(start_x, start_y)];
    while let Some((x, y)) = stack.pop() {
        let idx = (y * width + x) as usize;
        if visited[idx] {
            continue;
        }
        visited[idx] = true;

        // 8-way connectivity
        let neighbors = [
            (x.wrapping_sub(1), y.wrapping_sub(1)),
            (x, y.wrapping_sub(1)),
            (x + 1, y.wrapping_sub(1)),
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x.wrapping_sub(1), y + 1),
            (x, y + 1),
            (x + 1, y + 1),
        ];

        for &(nx, ny) in &neighbors {
            if nx < width && ny < height {
                let n_idx = (ny * width + nx) as usize;
                if !visited[n_idx] && img.get_pixel(nx, ny).0[0] == 0 {
                    stack.push((nx, ny));
                }
            }
        }
    }
}
