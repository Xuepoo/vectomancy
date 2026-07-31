use rayon::prelude::*;
use std::path::Path;
use tracing::{debug, info};
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

/// Row-major flat boolean grid, avoiding per-row heap allocations and the
/// double pointer indirection of a `Vec<Vec<bool>>`. The grid is padded by
/// 1 pixel on every side so the thinning/tracing neighbor scans never need
/// bounds checks against the original image edges.
struct FlatGrid {
    data: Vec<bool>,
    width: usize,
    height: usize,
}

impl FlatGrid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![false; width * height],
            width,
            height,
        }
    }

    #[inline(always)]
    fn get(&self, x: usize, y: usize) -> bool {
        self.data[y * self.width + x]
    }

    #[inline(always)]
    fn set(&mut self, x: usize, y: usize, v: bool) {
        self.data[y * self.width + x] = v;
    }
}

/// Zhang-Suen skeletonization thinning, parallelized across rows within each
/// half-iteration.
///
/// The algorithm alternates two sub-passes ("Step 1" / "Step 2") until no
/// pixel is deleted. Deletions within a single sub-pass are computed from a
/// read-only snapshot of the grid and only applied afterward, so every row's
/// candidate check in a sub-pass is independent of every other row's and can
/// run across all CPU cores. Only the deletions themselves, and the "did
/// anything change" decision that gates the next sub-pass, remain
/// sequential — which is inherent to the algorithm (each iteration depends
/// on the previous one's result).
fn zhang_suen_thinning(grid: &mut FlatGrid) {
    let width = grid.width;
    let height = grid.height;

    if width < 3 || height < 3 {
        return;
    }

    // Scans rows [1, height-1) in parallel, returning flagged pixels using the
    // given neighbor-based predicate. `p2..p9` follow the standard Zhang-Suen
    // clockwise neighbor numbering starting from north.
    let scan = |grid: &FlatGrid, sub_pass: u8| -> Vec<(usize, usize)> {
        let row_range = 1..height - 1;

        let scan_row = |y: usize| -> Vec<(usize, usize)> {
            let mut hits = Vec::new();
            for x in 1..width - 1 {
                if !grid.get(x, y) {
                    continue;
                }
                let p2 = grid.get(x, y - 1) as u8;
                let p3 = grid.get(x + 1, y - 1) as u8;
                let p4 = grid.get(x + 1, y) as u8;
                let p5 = grid.get(x + 1, y + 1) as u8;
                let p6 = grid.get(x, y + 1) as u8;
                let p7 = grid.get(x - 1, y + 1) as u8;
                let p8 = grid.get(x - 1, y) as u8;
                let p9 = grid.get(x - 1, y - 1) as u8;

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

                let keep = if sub_pass == 1 {
                    p2 * p4 * p6 == 0 && p4 * p6 * p8 == 0
                } else {
                    p2 * p4 * p8 == 0 && p2 * p6 * p8 == 0
                };
                if keep {
                    hits.push((x, y));
                }
            }
            hits
        };

        row_range
            .into_par_iter()
            .flat_map(scan_row)
            .collect::<Vec<_>>()
    };

    let mut changed = true;
    while changed {
        changed = false;

        let step1_hits = scan(grid, 1);
        if !step1_hits.is_empty() {
            changed = true;
            for (x, y) in step1_hits {
                grid.set(x, y, false);
            }
        }

        let step2_hits = scan(grid, 2);
        if !step2_hits.is_empty() {
            changed = true;
            for (x, y) in step2_hits {
                grid.set(x, y, false);
            }
        }
    }
}

/// Traces a single connected polyline starting from `(start_x, start_y)`,
/// consuming pixels from `visited` as it goes. Shared by both the endpoint
/// pass (open strokes) and the loop/remnant pass (closed strokes or isolated
/// components with no degree-1 pixel) in [`extract_paths`].
fn trace_from(
    grid: &FlatGrid,
    visited: &mut [bool],
    start_x: usize,
    start_y: usize,
) -> Vec<Point2D> {
    let width = grid.width;
    let height = grid.height;
    let idx = |x: usize, y: usize| y * width + x;

    let get_neighbors = |x: usize, y: usize| -> Vec<(usize, usize)> {
        let mut n = Vec::new();
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx as isize;
                let ny = y as isize + dy as isize;
                if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if grid.get(nx, ny) {
                        n.push((nx, ny));
                    }
                }
            }
        }
        n
    };

    let mut path = Vec::new();
    let mut curr_x = start_x;
    let mut curr_y = start_y;

    loop {
        visited[idx(curr_x, curr_y)] = true;
        // Coordinates are recorded relative to the original (unpadded) image;
        // the caller pads the grid by 1px on every side.
        path.push(Point2D {
            x: (curr_x as f64) - 1.0,
            y: (curr_y as f64) - 1.0,
        });

        let neighbors = get_neighbors(curr_x, curr_y);
        let mut next = None;

        for &(nx, ny) in &neighbors {
            if !visited[idx(nx, ny)] {
                next = Some((nx, ny));
                break;
            }
        }

        if let Some((nx, ny)) = next {
            curr_x = nx;
            curr_y = ny;
        } else {
            // Dead end: if this pixel also touches the pixel we arrived from
            // plus a second, already-visited neighbor, closing a short loop,
            // append it once more so the polyline reflects the closure.
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

    path
}

/// Extracts skeleton polylines from a thinned binary grid via graph
/// traversal. Endpoints (pixels with exactly one neighbor) are traced first
/// so open strokes are walked end-to-end; any pixels left over (closed
/// loops, or isolated blobs the endpoint pass never reached) are traced in a
/// second sweep. This matches the topology-aware extraction used by the
/// facade parser, replacing a naive greedy walk that stopped at the first
/// branch point and produced fragmented, noisy paths.
fn extract_paths(grid: &FlatGrid) -> Vec<Vec<Point2D>> {
    let width = grid.width;
    let height = grid.height;
    if width < 3 || height < 3 {
        return Vec::new();
    }

    let mut paths = Vec::new();
    let mut visited = vec![false; width * height];
    let idx = |x: usize, y: usize| y * width + x;

    let neighbor_count = |x: usize, y: usize| -> usize {
        let mut count = 0;
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx as isize;
                let ny = y as isize + dy as isize;
                if nx >= 0
                    && nx < width as isize
                    && ny >= 0
                    && ny < height as isize
                    && grid.get(nx as usize, ny as usize)
                {
                    count += 1;
                }
            }
        }
        count
    };

    // Pass 1: trace from endpoints (degree-1 pixels) so open strokes are
    // walked from tip to tip.
    let mut endpoints = Vec::new();
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if grid.get(x, y) && neighbor_count(x, y) == 1 {
                endpoints.push((x, y));
            }
        }
    }

    for (start_x, start_y) in endpoints {
        if visited[idx(start_x, start_y)] {
            continue;
        }
        let path = trace_from(grid, &mut visited, start_x, start_y);
        if path.len() > 1 {
            paths.push(path);
        }
    }

    // Pass 2: anything left over is either a closed loop or an isolated
    // component with no degree-1 pixel; trace those too.
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            if grid.get(x, y) && !visited[idx(x, y)] {
                let path = trace_from(grid, &mut visited, x, y);
                if path.len() > 1 {
                    paths.push(path);
                }
            }
        }
    }

    paths
}

#[allow(clippy::type_complexity)]
pub fn decode_raster_memory(
    bytes: &[u8],
    color: bool,
) -> Result<(Vec<StyledPath<Polyline>>, (u32, u32)), String> {
    info!("Processing raster image from memory in vectomancy-raster");
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let dimensions = (img.width(), img.height());

    // 1. Grayscale + Sobel edge detection
    debug!("Converting to grayscale");
    let gray = img.to_luma8();
    debug!("Applying Sobel edge detection");
    let gradients = sobel_gradients_parallel(&gray);

    // 2. Otsu binarization on the edge magnitudes
    debug!("Applying Otsu binarization");
    let edge_pixels: Vec<u8> = gradients.iter().map(|&v| v.min(255) as u8).collect();
    let (width, height) = dimensions;
    let edge_image = image::GrayImage::from_raw(width, height, edge_pixels)
        .expect("edge buffer matches image dimensions");
    let threshold = imageproc::contrast::otsu_level(&edge_image);
    info!("Otsu calculated threshold: {}", threshold);

    let padded_width = width as usize + 2;
    let padded_height = height as usize + 2;
    let mut grid = FlatGrid::new(padded_width, padded_height);

    for (x, y, pixel) in edge_image.enumerate_pixels() {
        if pixel.0[0] > threshold {
            grid.set(x as usize + 1, y as usize + 1, true);
        }
    }

    // 3. Zhang-Suen thinning to a 1px-wide skeleton
    debug!("Applying Zhang-Suen thinning");
    zhang_suen_thinning(&mut grid);

    // 4. Extract polylines via endpoint + loop-remnant graph traversal
    debug!("Extracting paths from thinned skeleton");
    let all_paths = extract_paths(&grid);
    info!(
        "Extracted {} skeleton paths (total {} points) from image",
        all_paths.len(),
        all_paths.iter().map(|p| p.len()).sum::<usize>()
    );

    let rgb_image = if color { Some(img.to_rgb8()) } else { None };

    let polylines: Vec<StyledPath<Polyline>> = all_paths
        .into_par_iter()
        .filter_map(|points| {
            if points.len() < 2 {
                return None;
            }
            let color_style = rgb_image.as_ref().and_then(|rgb| {
                let mut r_sum = 0u64;
                let mut g_sum = 0u64;
                let mut b_sum = 0u64;
                let mut count = 0u64;
                for pt in &points {
                    let x = pt.x.round() as i64;
                    let y = pt.y.round() as i64;
                    if x >= 0 && y >= 0 && (x as u32) < width && (y as u32) < height {
                        let pixel = rgb.get_pixel(x as u32, y as u32);
                        r_sum += pixel[0] as u64;
                        g_sum += pixel[1] as u64;
                        b_sum += pixel[2] as u64;
                        count += 1;
                    }
                }
                match (
                    r_sum.checked_div(count),
                    g_sum.checked_div(count),
                    b_sum.checked_div(count),
                ) {
                    (Some(r), Some(g), Some(b)) => {
                        Some(format!("#{:02x}{:02x}{:02x}", r as u8, g as u8, b as u8))
                    }
                    _ => None,
                }
            });

            Some(StyledPath {
                geometry: Polyline {
                    points,
                    closed: false,
                },
                color_style,
            })
        })
        .collect();

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
