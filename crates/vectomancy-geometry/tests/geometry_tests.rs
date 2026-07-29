use vectomancy_geometry::{
    chaikin_smooth, resample_by_arc_length, simplify_rdp, Point2D, Polyline,
};

#[test]
fn test_rdp_simplification() {
    let points = vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 1.0, y: 0.05 },
        Point2D { x: 2.0, y: 0.0 },
    ];
    let simplified = simplify_rdp(&points, 0.1);
    assert_eq!(simplified.len(), 2);
    assert_eq!(simplified[0], Point2D { x: 0.0, y: 0.0 });
    assert_eq!(simplified[1], Point2D { x: 2.0, y: 0.0 });
}

#[test]
fn test_chaikin_smooth() {
    let points = vec![
        Point2D { x: 0.0, y: 0.0 },
        Point2D { x: 4.0, y: 0.0 },
        Point2D { x: 4.0, y: 4.0 },
    ];
    let polyline = Polyline {
        points,
        closed: false,
    };
    let smoothed = chaikin_smooth(&polyline, 1);
    assert!(smoothed.points.len() > 3);
}

#[test]
fn test_arc_length_resampling() {
    let polyline = Polyline {
        points: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: 10.0, y: 0.0 }],
        closed: false,
    };
    let resampled = resample_by_arc_length(&polyline, 1.0);
    assert!(resampled.points.len() >= 10);
}
