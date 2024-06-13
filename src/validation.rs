use anyhow::Result;
use geo::FrechetDistance;
use geo::OutlierDetection;
use geo::{
    coord, DensifyHaversine, LineInterpolatePoint, LineString, Point, RemoveRepeatedPoints,
    Simplify,
};

use crate::journeys::GpsTrace;

pub fn validation(t1: GpsTrace, t2: GpsTrace) -> Result<(geo::LineString, geo::LineString)> {
    let line_strings: Vec<LineString<f64>> = [t1, t2]
        .iter()
        .map(|t| {
            let coords: Vec<Point> = t.points.iter().map(Point::from).collect();

            // remove outliers

            let outlier_scores = coords.outliers(2);
            let filtered_coords: Vec<_> = coords
                .iter()
                .zip(outlier_scores.iter())
                .filter(|(_, &score)| score <= 1.0) // Adjust threshold as needed
                .map(|(&point, _)| point)
                .collect();

            let mut line_string = LineString(
                filtered_coords
                    .iter()
                    .map(|p| coord! { x: p.x(), y: p.y() })
                    .collect::<Vec<_>>(),
            );

            println!("Interpolate trajectories...");
            line_string.densify_haversine(0.1);
            line_string.line_interpolate_point(0.1);

            println!("Removing outliers...");
            line_string.remove_repeated_points_mut();

            line_string
        })
        .collect();

    let line_string1 = &line_strings[0];
    let line_string2 = &line_strings[1];

    println!("Frechet distance...");
    let dist = line_string1.frechet_distance(line_string2);
    println!("Frechet distance: {}", dist);

    println!("Simplify...");
    line_string1.simplify(&0.1);
    line_string2.simplify(&0.1);

    Ok((line_string1.clone(), line_string2.clone()))
}
