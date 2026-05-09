use crate::models::{BezierSegment, Point2D, SplineEquation};

pub fn build_splines(segments: &[BezierSegment]) -> Vec<SplineEquation> {
    let mut equations = Vec::new();
    let mut current_t = 0.0;
    let mut current_point = Point2D { x: 0.0, y: 0.0 };
    let mut first_point = Point2D { x: 0.0, y: 0.0 };

    for segment in segments {
        match segment {
            BezierSegment::MoveTo(p) => {
                current_point = *p;
                first_point = *p;
            }
            BezierSegment::LineTo(p) => {
                let x0 = current_point.x;
                let y0 = current_point.y;
                let x1 = p.x;
                let y1 = p.y;

                let x_poly = vec![x0, x1 - x0, 0.0, 0.0];
                let y_poly = vec![y0, y1 - y0, 0.0, 0.0];

                equations.push(SplineEquation {
                    start_t: current_t,
                    end_t: current_t + 1.0,
                    x_poly,
                    y_poly,
                });

                current_point = *p;
                current_t += 1.0;
            }
            BezierSegment::QuadraticTo(p1, p2) => {
                let x0 = current_point.x;
                let y0 = current_point.y;
                let x1 = p1.x;
                let y1 = p1.y;
                let x2 = p2.x;
                let y2 = p2.y;

                let x_poly = vec![x0, 2.0 * (x1 - x0), x0 - 2.0 * x1 + x2, 0.0];
                let y_poly = vec![y0, 2.0 * (y1 - y0), y0 - 2.0 * y1 + y2, 0.0];

                equations.push(SplineEquation {
                    start_t: current_t,
                    end_t: current_t + 1.0,
                    x_poly,
                    y_poly,
                });

                current_point = *p2;
                current_t += 1.0;
            }
            BezierSegment::CubicTo(p1, p2, p3) => {
                let x0 = current_point.x;
                let y0 = current_point.y;
                let x1 = p1.x;
                let y1 = p1.y;
                let x2 = p2.x;
                let y2 = p2.y;
                let x3 = p3.x;
                let y3 = p3.y;

                let x_poly = vec![
                    x0,
                    3.0 * (x1 - x0),
                    3.0 * (x0 - 2.0 * x1 + x2),
                    -x0 + 3.0 * x1 - 3.0 * x2 + x3,
                ];
                let y_poly = vec![
                    y0,
                    3.0 * (y1 - y0),
                    3.0 * (y0 - 2.0 * y1 + y2),
                    -y0 + 3.0 * y1 - 3.0 * y2 + y3,
                ];

                equations.push(SplineEquation {
                    start_t: current_t,
                    end_t: current_t + 1.0,
                    x_poly,
                    y_poly,
                });

                current_point = *p3;
                current_t += 1.0;
            }
            BezierSegment::Close => {
                let x0 = current_point.x;
                let y0 = current_point.y;
                let x1 = first_point.x;
                let y1 = first_point.y;

                let x_poly = vec![x0, x1 - x0, 0.0, 0.0];
                let y_poly = vec![y0, y1 - y0, 0.0, 0.0];

                equations.push(SplineEquation {
                    start_t: current_t,
                    end_t: current_t + 1.0,
                    x_poly,
                    y_poly,
                });

                current_point = first_point;
                current_t += 1.0;
            }
        }
    }

    equations
}

pub fn sample_subpaths(segments: &[BezierSegment], points_per_segment: usize) -> Vec<Vec<Point2D>> {
    let mut subpaths = Vec::new();
    let mut current_path = Vec::new();
    let mut current_point = Point2D { x: 0.0, y: 0.0 };
    let mut first_point = Point2D { x: 0.0, y: 0.0 };

    for segment in segments {
        match segment {
            BezierSegment::MoveTo(p) => {
                if !current_path.is_empty() {
                    subpaths.push(current_path);
                    current_path = Vec::new();
                }
                current_point = *p;
                first_point = *p;
                current_path.push(*p);
            }
            BezierSegment::LineTo(p) => {
                for i in 1..=points_per_segment {
                    let t = i as f64 / points_per_segment as f64;
                    let x = current_point.x + (p.x - current_point.x) * t;
                    let y = current_point.y + (p.y - current_point.y) * t;
                    current_path.push(Point2D { x, y });
                }
                current_point = *p;
            }
            BezierSegment::QuadraticTo(p1, p2) => {
                for i in 1..=points_per_segment {
                    let t = i as f64 / points_per_segment as f64;
                    let u = 1.0 - t;
                    let x = u * u * current_point.x + 2.0 * u * t * p1.x + t * t * p2.x;
                    let y = u * u * current_point.y + 2.0 * u * t * p1.y + t * t * p2.y;
                    current_path.push(Point2D { x, y });
                }
                current_point = *p2;
            }
            BezierSegment::CubicTo(p1, p2, p3) => {
                for i in 1..=points_per_segment {
                    let t = i as f64 / points_per_segment as f64;
                    let u = 1.0 - t;
                    let x = u * u * u * current_point.x
                        + 3.0 * u * u * t * p1.x
                        + 3.0 * u * t * t * p2.x
                        + t * t * t * p3.x;
                    let y = u * u * u * current_point.y
                        + 3.0 * u * u * t * p1.y
                        + 3.0 * u * t * t * p2.y
                        + t * t * t * p3.y;
                    current_path.push(Point2D { x, y });
                }
                current_point = *p3;
            }
            BezierSegment::Close => {
                for i in 1..=points_per_segment {
                    let t = i as f64 / points_per_segment as f64;
                    let x = current_point.x + (first_point.x - current_point.x) * t;
                    let y = current_point.y + (first_point.y - current_point.y) * t;
                    current_path.push(Point2D { x, y });
                }
                current_point = first_point;
            }
        }
    }
    if !current_path.is_empty() {
        subpaths.push(current_path);
    }
    subpaths
}

pub fn sample_segments(segments: &[BezierSegment], points_per_segment: usize) -> Vec<Point2D> {
    let equations = build_splines(segments);
    let mut points = Vec::new();

    for eq in equations {
        for i in 0..points_per_segment {
            let t = i as f64 / points_per_segment as f64;
            let x = eq.x_poly.first().copied().unwrap_or(0.0)
                + eq.x_poly.get(1).copied().unwrap_or(0.0) * t
                + eq.x_poly.get(2).copied().unwrap_or(0.0) * t * t
                + eq.x_poly.get(3).copied().unwrap_or(0.0) * t * t * t;
            let y = eq.y_poly.first().copied().unwrap_or(0.0)
                + eq.y_poly.get(1).copied().unwrap_or(0.0) * t
                + eq.y_poly.get(2).copied().unwrap_or(0.0) * t * t
                + eq.y_poly.get(3).copied().unwrap_or(0.0) * t * t * t;
            points.push(Point2D { x, y });
        }
    }
    points
}

pub fn fit_cubic_bezier(points: &[Point2D]) -> Vec<crate::models::BezierSegment> {
    if points.len() < 3 {
        let mut segments = Vec::new();
        if let Some(first) = points.first() {
            segments.push(BezierSegment::MoveTo(*first));
            for p in points.iter().skip(1) {
                segments.push(BezierSegment::LineTo(*p));
            }
        }
        return segments;
    }

    let mut segments = Vec::new();
    segments.push(BezierSegment::MoveTo(points[0]));

    for i in 0..points.len() - 1 {
        let p0 = if i == 0 { points[0] } else { points[i - 1] };
        let p1 = points[i];
        let p2 = points[i + 1];
        let p3 = if i + 2 < points.len() {
            points[i + 2]
        } else {
            points[i + 1]
        };

        let cp1 = Point2D {
            x: p1.x + (p2.x - p0.x) / 6.0,
            y: p1.y + (p2.y - p0.y) / 6.0,
        };
        let cp2 = Point2D {
            x: p2.x - (p3.x - p1.x) / 6.0,
            y: p2.y - (p3.y - p1.y) / 6.0,
        };

        segments.push(BezierSegment::CubicTo(cp1, cp2, p2));
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_cubic_bezier() {
        let points = vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 10.0, y: 0.0 },
            Point2D { x: 10.0, y: 10.0 },
        ];
        let segments = fit_cubic_bezier(&points);
        assert_eq!(segments.len(), 3);
        if let BezierSegment::MoveTo(p) = segments[0] {
            assert_eq!(p.x, 0.0);
            assert_eq!(p.y, 0.0);
        } else {
            panic!("Expected MoveTo");
        }
        if let BezierSegment::CubicTo(cp1, cp2, p2) = segments[1] {
            assert_eq!(p2.x, 10.0);
            assert_eq!(p2.y, 0.0);
        } else {
            panic!("Expected CubicTo");
        }
    }
}
