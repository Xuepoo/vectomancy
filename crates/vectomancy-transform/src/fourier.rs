use crate::models::FourierTerm;
use rustfft::{num_complex::Complex, FftPlanner};
use std::cell::RefCell;
use vectomancy_geometry::Point2D;

thread_local! {
    static FFT_PLANNER: RefCell<FftPlanner<f64>> = RefCell::new(FftPlanner::new());
}

pub fn perform_fft(
    points: &[Point2D],
    terms: usize,
    use_gpu: bool,
    adaptive: bool,
    energy_threshold: f64,
) -> Result<Vec<FourierTerm>, String> {
    if use_gpu {
        // Fallback or GPU path
    }

    if points.is_empty() {
        return Ok(Vec::new());
    }

    let n = points.len();
    let mut buffer: Vec<Complex<f64>> = points.iter().map(|p| Complex::new(p.x, p.y)).collect();

    FFT_PLANNER.with(|planner| {
        let fft = planner.borrow_mut().plan_fft_forward(n);
        fft.process(&mut buffer);
    });

    let n_f64 = n as f64;
    let mut all_terms = Vec::with_capacity(n);

    for (i, val) in buffer.iter().enumerate() {
        let freq = if i <= n / 2 {
            i as f64
        } else {
            (i as f64) - n_f64
        };

        let magnitude = val.norm() / n_f64;
        let phase = val.arg();

        all_terms.push(FourierTerm {
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

    let mut terms_vec = Vec::new();

    if adaptive {
        let total_energy: f64 = all_terms.iter().map(|t| t.amplitude * t.amplitude).sum();
        let mut current_energy = 0.0;
        let target_energy = total_energy * energy_threshold;

        for term in all_terms {
            current_energy += term.amplitude * term.amplitude;
            terms_vec.push(term);

            if current_energy >= target_energy || terms_vec.len() >= terms {
                break;
            }
        }
    } else {
        terms_vec = all_terms.into_iter().take(terms).collect();
    }

    Ok(terms_vec)
}
