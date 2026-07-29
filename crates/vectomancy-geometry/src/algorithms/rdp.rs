use crate::types::Point2D;

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

/// Ramer-Douglas-Peucker (RDP) polyline simplification algorithm.
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
