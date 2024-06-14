pub fn normalize_frechet_distance(frechet_distance: f64) -> f64 {
    let max_distance = 10.0;

    let normalized_distance = frechet_distance / max_distance;

    1.0 - normalized_distance.clamp(0.0, 1.0)
}
