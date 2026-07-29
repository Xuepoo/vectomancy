use crate::types::{Point2D, Polyline};

/// Resample a polyline uniformly along its arc-length by `step_size`.
pub fn resample_by_arc_length(polyline: &Polyline, step_size: f64) -> Polyline {
    if polyline.points.len() < 2 || step_size <= 0.0 {
        return polyline.clone();
    }

    let points = &polyline.points;
    let mut resampled = vec![points[0]];

    let mut current_segment_index = 0;
    let mut current_segment_offset = 0.0;

    while current_segment_index < points.len() - 1 {
        let p0 = points[current_segment_index];
        let p1 = points[current_segment_index + 1];
        let seg_len = p0.distance(&p1);

        if seg_len == 0.0 {
            current_segment_index += 1;
            current_segment_offset = 0.0;
            continue;
        }

        let dist_remaining_in_segment = seg_len - current_segment_offset;

        if dist_remaining_in_segment >= step_size {
            current_segment_offset += step_size;
            let t = current_segment_offset / seg_len;
            let new_pt = Point2D {
                x: p0.x + t * (p1.x - p0.x),
                y: p0.y + t * (p1.y - p0.y),
            };
            resampled.push(new_pt);
        } else {
            current_segment_index += 1;
            current_segment_offset = 0.0;
        }
    }

    if let Some(&last) = points.last() {
        if resampled.last() != Some(&last) {
            resampled.push(last);
        }
    }

    Polyline {
        points: resampled,
        closed: polyline.closed,
    }
}
