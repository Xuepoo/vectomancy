use kiddo::KdTree;
use tracing::debug;
use vectomancy_geometry::Point2D;

pub fn solve_tsp_nearest_neighbor(points: Vec<Point2D>) -> Vec<Point2D> {
    if points.is_empty() {
        return Vec::new();
    }

    debug!("Solving TSP (Nearest Neighbor) for {} points", points.len());
    let mut tree: KdTree<f64, 2> = KdTree::new();

    for (i, p) in points.iter().enumerate() {
        tree.add(&[p.x, p.y], i as u64);
    }

    let mut ordered = Vec::with_capacity(points.len());

    let mut current_idx = 0;
    let mut current_point = points[current_idx];
    tree.remove(&[current_point.x, current_point.y], current_idx as u64);
    ordered.push(current_point);

    for _ in 1..points.len() {
        let nearest =
            tree.nearest_one::<kiddo::SquaredEuclidean>(&[current_point.x, current_point.y]);
        current_idx = nearest.item as usize;
        current_point = points[current_idx];

        tree.remove(&[current_point.x, current_point.y], current_idx as u64);
        ordered.push(current_point);
    }

    debug!("Applying 2-Opt optimization");
    let mut improvement = true;
    let n = ordered.len();
    let max_iterations = if n > 5000 { 1 } else { 10 };
    let mut iter_count = 0;

    while improvement && iter_count < max_iterations {
        improvement = false;
        iter_count += 1;

        for i in 0..n - 1 {
            for k in (i + 2)..n {
                if i == 0 && k == n - 1 {
                    continue;
                }

                let p1 = ordered[i];
                let p2 = ordered[i + 1];
                let p3 = ordered[k];
                let p4 = if k + 1 < n {
                    ordered[k + 1]
                } else {
                    ordered[0]
                };

                let current_dist = p1.distance(&p2) + p3.distance(&p4);
                let new_dist = p1.distance(&p3) + p2.distance(&p4);

                if new_dist < current_dist {
                    ordered[i + 1..=k].reverse();
                    improvement = true;
                }
            }
        }
    }

    ordered
}
