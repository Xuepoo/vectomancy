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
