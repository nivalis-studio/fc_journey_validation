use crate::traces::Trace;
use geo::{HaversineDistance, Point};

fn bounding_box(points: &[Point<f64>]) -> (Point<f64>, Point<f64>) {
    let min_x = points.iter().map(|p| p.x()).fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|p| p.x())
        .fold(f64::NEG_INFINITY, f64::max);

    let min_y = points.iter().map(|p| p.y()).fold(f64::INFINITY, f64::min);
    let max_y = points
        .iter()
        .map(|p| p.y())
        .fold(f64::NEG_INFINITY, f64::max);

    (Point::new(min_x, min_y), Point::new(max_x, max_y))
}

pub fn normalize_frechet_distance(t1: &Trace, t2: &Trace, frechet_distance: f64) -> f64 {
    let all_points: Vec<Point<f64>> = t1.points().chain(t2.points()).collect();
    let (min_point, max_point) = bounding_box(&all_points);

    let max_distance = min_point.haversine_distance(&max_point);

    let normalized_distance = frechet_distance / max_distance;

    1.0 - normalized_distance.clamp(0.0, 1.0)
}
