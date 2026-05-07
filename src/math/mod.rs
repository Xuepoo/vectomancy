pub mod spline;

use crate::error::VectomancyError;
use crate::models::{MathExpressionAST, Point2D};
use rustfft::{num_complex::Complex, FftPlanner};
use tracing::info;

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

pub fn solve_tsp_nearest_neighbor(points: Vec<Point2D>) -> Vec<Point2D> {
    if points.is_empty() {
        return Vec::new();
    }

    info!("Solving TSP (Nearest Neighbor) for {} points", points.len());
    let mut unvisited = points;
    let mut ordered = Vec::with_capacity(unvisited.len());

    // Start with the first point
    ordered.push(unvisited.remove(0));

    while !unvisited.is_empty() {
        let last = ordered.last().unwrap();

        let (best_idx, _) = unvisited
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = (last.x - a.x).powi(2) + (last.y - a.y).powi(2);
                let dist_b = (last.x - b.x).powi(2) + (last.y - b.y).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        ordered.push(unvisited.remove(best_idx));
    }

    // TODO: Implement 2-Opt optimization here for better results

    ordered
}

pub fn perform_fft(points: &[Point2D], terms: usize) -> Result<MathExpressionAST, VectomancyError> {
    info!("Performing FFT. Terms: {}", terms);
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(points.len());

    // Convert points to complex numbers
    let mut buffer: Vec<Complex<f64>> = points
        .iter()
        .map(|p| Complex { re: p.x, im: p.y })
        .collect();

    fft.process(&mut buffer);

    let mut terms_vec = Vec::new();

    let n = buffer.len() as f64;
    // Extract top N frequencies
    for (i, val) in buffer.iter().take(terms.min(buffer.len())).enumerate() {
        let freq = if i <= buffer.len() / 2 {
            i as f64
        } else {
            (i as f64) - n
        };
        let magnitude = val.norm() / n;
        let phase = val.arg();

        if magnitude > 0.001 {
            terms_vec.push(crate::models::FourierTerm {
                amplitude: magnitude,
                frequency: freq,
                phase: phase,
            });
        }
    }

    Ok(MathExpressionAST::Fourier { terms: terms_vec })
}
