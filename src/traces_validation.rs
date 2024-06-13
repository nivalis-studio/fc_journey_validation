use crate::journeys::GpsTrace;
use anyhow::Result;
use geo::ChaikinSmoothing;
use geo::FrechetDistance;
use geo::HaversineLength;
use geo::OutlierDetection;
use geo::{
    coord, DensifyHaversine, LineInterpolatePoint, LineString, Point, RemoveRepeatedPoints,
    Simplify,
};

use crate::traces_to_geojson::traces_to_geojson;

pub fn traces_validation(t1: GpsTrace, t2: GpsTrace) -> Result<(geo::LineString, geo::LineString)> {
    let line_strings: Vec<LineString<f64>> = [t1, t2]
        .iter()
        .map(|t| {
            let coords: Vec<Point> = t.points.iter().map(Point::from).collect();

            let filtered_coords: Vec<_> = coords
                .iter()
                .zip(coords.outliers(3).iter())
                .filter(|(_, &score)| score <= 1.0) // Adjust threshold as needed
                .map(|(&point, _)| point)
                .collect();

            let line_string = LineString(
                filtered_coords
                    .iter()
                    .map(|p| coord! { x: p.x(), y: p.y() })
                    .collect::<Vec<_>>(),
            );

            line_string.densify_haversine(0.1);
            line_string.line_interpolate_point(0.1);

            line_string
        })
        .collect();

    let line_string1 = &line_strings[0];
    let line_string2 = &line_strings[1];

    let dist = line_string1.frechet_distance(line_string2);
    println!("Frechet distance: {}", dist);

    line_string1.remove_repeated_points();
    line_string2.remove_repeated_points();

    line_string1.chaikin_smoothing(3);
    line_string2.chaikin_smoothing(3);

    line_string1.simplify(&0.1);
    line_string2.simplify(&0.1);

    let line1_length = line_string1.haversine_length();
    let line2_length = line_string2.haversine_length();

    println!("Line 1 length: {} meters", line1_length);
    println!("Line 2 length: {} meters", line2_length);

    traces_to_geojson(line_string1, line_string2)?;

    Ok((line_string1.clone(), line_string2.clone()))
}
