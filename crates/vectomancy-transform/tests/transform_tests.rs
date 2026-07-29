use vectomancy_geometry::Point2D;
use vectomancy_transform::{build_splines, perform_fft, solve_tsp_nearest_neighbor, BezierSegment};

#[test]
fn test_tsp_solver() {
    let points = vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 10.0, y: 0.0 },
        Point2D { x: 2.0, y: 0.0 },
    ];
    let ordered = solve_tsp_nearest_neighbor(points);
    assert_eq!(ordered.len(), 3);
    assert_eq!(ordered[0], Point2D { x: 0.0, y: 0.0 });
    assert_eq!(ordered[1], Point2D { x: 2.0, y: 0.0 });
    assert_eq!(ordered[2], Point2D { x: 10.0, y: 0.0 });
}

#[test]
fn test_fft_transform() {
    let points = (0..64)
        .map(|i| Point2D {
            x: (i as f64).cos(),
            y: (i as f64).sin(),
        })
        .collect::<Vec<_>>();
    let terms = perform_fft(&points, 10, false, false, 0.99).unwrap();
    assert_eq!(terms.len(), 10);
}

#[test]
fn test_build_splines() {
    let segments = vec![
        BezierSegment::MoveTo(Point2D { x: 0.0, y: 0.0 }),
        BezierSegment::LineTo(Point2D { x: 10.0, y: 10.0 }),
    ];
    let splines = build_splines(&segments, true);
    assert_eq!(splines.len(), 1);
}
