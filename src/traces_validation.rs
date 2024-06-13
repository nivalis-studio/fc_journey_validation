use crate::journeys::GpsTrace;
use crate::journeys::Trace;
use anyhow::Result;
use geo::FrechetDistance;
use geo::HaversineLength;

pub fn traces_validation(t1: GpsTrace, t2: GpsTrace) -> Result<(Trace, Trace)> {
    let line_string1: Trace = t1.into();
    let line_string2: Trace = t2.into();

    let dist = line_string1
        .as_ref()
        .frechet_distance(line_string2.as_ref());
    println!("Frechet distance: {}", dist);

    let line_string1 = line_string1.simplified();
    let line_string2 = line_string2.simplified();

    let line1_length = line_string1.as_ref().haversine_length();
    let line2_length = line_string2.as_ref().haversine_length();

    println!("Line 1 length: {} meters", line1_length);
    println!("Line 2 length: {} meters", line2_length);

    // traces_to_geojson(line_string1, line_string2)?;

    Ok((line_string1, line_string2))
}
