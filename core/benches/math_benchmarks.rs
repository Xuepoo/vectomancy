use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::f64::consts::PI;
use vectomancy::math::{simplify_rdp, solve_tsp_nearest_neighbor};
use vectomancy::models::Point2D;

fn generate_circle_points(n: usize) -> Vec<Point2D> {
    let mut points = Vec::with_capacity(n);
    for i in 0..n {
        let theta = 2.0 * PI * (i as f64) / (n as f64);
        points.push(Point2D {
            x: theta.cos() * 100.0,
            y: theta.sin() * 100.0,
        });
    }
    points
}

pub fn bench_math_pipeline(c: &mut Criterion) {
    let points = generate_circle_points(2000); // Simulate an image contour

    let mut group = c.benchmark_group("Math Pipeline");

    group.bench_function("RDP Simplification (epsilon=1.0)", |b| {
        b.iter(|| simplify_rdp(black_box(&points), black_box(1.0)))
    });

    let reduced_points = simplify_rdp(&points, 1.0);
    group.bench_function("TSP Nearest Neighbor", |b| {
        b.iter(|| solve_tsp_nearest_neighbor(black_box(reduced_points.clone())))
    });

    group.finish();
}

criterion_group!(benches, bench_math_pipeline);
criterion_main!(benches);
