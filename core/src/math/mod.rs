pub mod spline;
#[cfg(not(target_arch = "wasm32"))]
pub mod wgpu_math;

use crate::error::VectomancyError;
use crate::models::Point2D;
use rustfft::{num_complex::Complex, FftPlanner};
use std::cell::RefCell;
use tracing::debug;

thread_local! {
    static FFT_PLANNER: RefCell<FftPlanner<f64>> = RefCell::new(FftPlanner::new());
}

fn perpendicular_distance(pt: Point2D, line_start: Point2D, line_end: Point2D) -> f64 {
    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;

    let mag = (dx * dx + dy * dy).sqrt();
    if mag == 0.0 {
        return ((pt.x - line_start.x).powi(2) + (pt.y - line_start.y).powi(2)).sqrt();
    }

    ((line_end.x - line_start.x) * (line_start.y - pt.y)
        - (line_start.x - pt.x) * (line_end.y - line_start.y))
        .abs()
        / mag
}

pub fn simplify_rdp(points: &[Point2D], epsilon: f64) -> Vec<Point2D> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut dmax = 0.0;
    let mut index = 0;
    let end = points.len() - 1;

    for i in 1..end {
        let d = perpendicular_distance(points[i], points[0], points[end]);
        if d > dmax {
            index = i;
            dmax = d;
        }
    }

    let mut result = Vec::new();
    if dmax > epsilon {
        let mut rec_results1 = simplify_rdp(&points[0..=index], epsilon);
        let mut rec_results2 = simplify_rdp(&points[index..=end], epsilon);

        rec_results1.pop(); // Remove the shared point
        result.append(&mut rec_results1);
        result.append(&mut rec_results2);
    } else {
        result.push(points[0]);
        result.push(points[end]);
    }

    result
}

use kiddo::KdTree;

pub fn solve_tsp_nearest_neighbor(points: Vec<Point2D>) -> Vec<Point2D> {
    if points.is_empty() {
        return Vec::new();
    }

    debug!("Solving TSP (Nearest Neighbor) for {} points", points.len());
    let mut tree: KdTree<f64, 2> = KdTree::new();

    for (i, p) in points.iter().enumerate() {
        tree.add(&[p.x, p.y], i as u64);
    }

    let mut ordered = Vec::with_capacity(points.len());

    // Start with the first point
    let mut current_idx = 0;
    let mut current_point = points[current_idx];
    tree.remove(&[current_point.x, current_point.y], current_idx as u64);
    ordered.push(current_point);

    for _ in 1..points.len() {
        let nearest =
            tree.nearest_one::<kiddo::SquaredEuclidean>(&[current_point.x, current_point.y]);
        current_idx = nearest.item as usize;
        current_point = points[current_idx];

        tree.remove(&[current_point.x, current_point.y], current_idx as u64);
        ordered.push(current_point);
    }

    debug!("Applying 2-Opt optimization");
    let mut improvement = true;
    let n = ordered.len();
    let max_iterations = if n > 5000 { 1 } else { 10 };
    let mut iter_count = 0;

    while improvement && iter_count < max_iterations {
        improvement = false;
        iter_count += 1;
        for i in 0..n.saturating_sub(1) {
            for j in i + 2..n {
                let d_i = ordered[i];
                let d_i1 = ordered[i + 1];
                let d_j = ordered[j];
                let d_j1 = ordered[(j + 1) % n];

                let dist_i_i1 = (d_i.x - d_i1.x).powi(2) + (d_i.y - d_i1.y).powi(2);
                let dist_j_j1 = (d_j.x - d_j1.x).powi(2) + (d_j.y - d_j1.y).powi(2);

                let dist_i_j = (d_i.x - d_j.x).powi(2) + (d_i.y - d_j.y).powi(2);
                let dist_i1_j1 = (d_i1.x - d_j1.x).powi(2) + (d_i1.y - d_j1.y).powi(2);

                if dist_i_j + dist_i1_j1 < dist_i_i1 + dist_j_j1 {
                    ordered[i + 1..=j].reverse();
                    improvement = true;
                }
            }
        }
    }

    ordered
}

pub fn perform_fft(
    points: &[Point2D],
    terms: usize,
    #[allow(unused_variables)] use_gpu: bool,
) -> Result<Vec<crate::models::FourierTerm>, VectomancyError> {
    #[cfg(not(target_arch = "wasm32"))]
    if use_gpu {
        match wgpu_math::perform_fft_gpu(points, terms) {
            Ok(res) => return Ok(res),
            Err(e) => {
                tracing::warn!("GPU FFT failed: {}. Falling back to CPU.", e);
            }
        }
    }

    debug!("Performing FFT. Terms: {}", terms);
    let fft = FFT_PLANNER.with(|planner| planner.borrow_mut().plan_fft_forward(points.len()));

    // Convert points to complex numbers
    let mut buffer: Vec<Complex<f64>> = points
        .iter()
        .map(|p| Complex { re: p.x, im: p.y })
        .collect();

    fft.process(&mut buffer);

    let mut terms_vec = Vec::new();

    let n = buffer.len() as f64;

    let mut all_terms = Vec::with_capacity(buffer.len());
    for (i, val) in buffer.iter().enumerate() {
        let freq = if i <= buffer.len() / 2 {
            i as f64
        } else {
            (i as f64) - n
        };
        let magnitude = val.norm() / n;
        let phase = val.arg();

        all_terms.push(crate::models::FourierTerm {
            amplitude: magnitude,
            frequency: freq,
            phase,
        });
    }

    all_terms.sort_by(|a, b| {
        b.amplitude
            .partial_cmp(&a.amplitude)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for term in all_terms
        .into_iter()
        .filter(|t| t.amplitude > 0.001)
        .take(terms)
    {
        terms_vec.push(term);
    }

    Ok(terms_vec)
}

pub fn chaikin_smooth(points: &[Point2D], iterations: usize) -> Vec<Point2D> {
    if points.len() < 3 || iterations == 0 {
        return points.to_vec();
    }

    let mut current = points.to_vec();
    for _ in 0..iterations {
        let mut next = Vec::with_capacity(current.len() * 2);
        next.push(current[0]); // Keep the first point
        for i in 0..current.len() - 1 {
            let p0 = current[i];
            let p1 = current[i + 1];

            let q0 = Point2D {
                x: 0.75 * p0.x + 0.25 * p1.x,
                y: 0.75 * p0.y + 0.25 * p1.y,
            };
            let q1 = Point2D {
                x: 0.25 * p0.x + 0.75 * p1.x,
                y: 0.25 * p0.y + 0.75 * p1.y,
            };

            next.push(q0);
            next.push(q1);
        }
        next.push(current[current.len() - 1]); // Keep the last point
        current = next;
    }
    current
}

pub fn perform_fft_batch(
    paths: &[&[Point2D]],
    terms: usize,
    #[allow(unused_variables)] use_gpu: bool,
) -> Result<Vec<Vec<crate::models::FourierTerm>>, VectomancyError> {
    #[cfg(not(target_arch = "wasm32"))]
    if use_gpu {
        match wgpu_math::perform_fft_batch_gpu(paths, terms) {
            Ok(res) => return Ok(res),
            Err(e) => {
                tracing::warn!("GPU Batch FFT failed: {}. Falling back to CPU.", e);
            }
        }
    }

    debug!(
        "Performing Batch FFT on CPU. Paths: {}, Terms: {}",
        paths.len(),
        terms
    );
    let mut all_results = Vec::with_capacity(paths.len());
    for points in paths {
        all_results.push(perform_fft(points, terms, false)?);
    }
    Ok(all_results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaikin_smooth() {
        let points = vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 10.0, y: 0.0 },
            Point2D { x: 10.0, y: 10.0 },
        ];
        let smoothed = chaikin_smooth(&points, 1);
        assert_eq!(smoothed.len(), 6);
        assert_eq!(smoothed[0], Point2D { x: 0.0, y: 0.0 });
        assert_eq!(smoothed[1], Point2D { x: 2.5, y: 0.0 });
        assert_eq!(smoothed[2], Point2D { x: 7.5, y: 0.0 });
        assert_eq!(smoothed[3], Point2D { x: 10.0, y: 2.5 });
        assert_eq!(smoothed[4], Point2D { x: 10.0, y: 7.5 });
        assert_eq!(smoothed[5], Point2D { x: 10.0, y: 10.0 });
    }
}
