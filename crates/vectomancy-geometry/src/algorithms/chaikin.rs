use crate::types::{Point2D, Polyline};

/// Chaikin corner-cutting algorithm for polyline smoothing.
pub fn chaikin_smooth(polyline: &Polyline, iterations: usize) -> Polyline {
    let points = &polyline.points;
    if points.len() < 3 || iterations == 0 {
        return polyline.clone();
    }

    let mut current = points.to_vec();
    for _ in 0..iterations {
        let mut next = Vec::with_capacity(current.len() * 2);

        if polyline.closed {
            let n = current.len();
            for i in 0..n {
                let p0 = current[i];
                let p1 = current[(i + 1) % n];

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
        } else {
            next.push(current[0]);
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
            next.push(current[current.len() - 1]);
        }
        current = next;
    }

    Polyline {
        points: current,
        closed: polyline.closed,
    }
}
