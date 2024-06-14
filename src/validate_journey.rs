use crate::traces::{Trace, TracesPair};
use crate::Result;
use crate::{journeys::Journey, validate_traces};
use geo::{Closest, Coord, HaversineLength, LineString, Point};
use geo::{HaversineClosestPoint, HaversineDistance};

const MIN_DISTANCE_IN_METERS: u16 = 1000;
const MAX_DISTANCE_IN_METERS: u32 = 80_000;

#[derive(Debug)]
pub enum ValidateReturnError {
    Error { success: bool, reason: &'static str },
}

#[derive(Debug)]
pub enum ValidateReturnSuccess<T> {
    Success { success: bool, data: T },
}

#[derive(Debug)]
pub enum ValidateReturn<T> {
    Error(ValidateReturnError),
    Success(ValidateReturnSuccess<T>),
}

pub fn validate_journey(journey: Journey) -> Result<ValidateReturn<()>> {
    let traces = journey.validate()?;

    let TracesPair(driver_trace, passenger_trace) = traces.validate()?;

    let t1: Trace = driver_trace.clone().into();
    let t2: Trace = passenger_trace.into();

    let validate_traces_res = validate_traces(&t1, &t2);
    println!("Frechet distance: {:?}", validate_traces_res.unwrap());

    let mut common_coords: Vec<Coord<f64>> = Vec::new();

    for point1 in driver_trace.points.iter() {
        let point1: Point<f64> = point1.into();
        let point2: Closest<f64> = t2.haversine_closest_point(&point1);

        let point2: Point<f64> = match point2 {
            Closest::SinglePoint(point) => point,
            Closest::Intersection(intersection) => intersection,
            Closest::Indeterminate => continue,
        };

        let dist = point1.haversine_distance(&point2);

        if dist < 1000.0 {
            common_coords.push(Coord {
                x: point1.x(),
                y: point1.y(),
            })
        }
    }

    let common_line_string: LineString = LineString::new(common_coords);
    let distance = common_line_string.haversine_length();

    println!("Common distance: {:?}", distance);

    if distance < MIN_DISTANCE_IN_METERS as f64 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "distance too short",
        }));
    }

    if distance > MAX_DISTANCE_IN_METERS as f64 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "distance too long",
        }));
    }

    let lines: Vec<LineString> = [t1, t2]
        .iter()
        .map(|trace| {
            let trace = trace.simplified();
            let length = trace.haversine_length();
            println!("Line length: {} meters", length);

            trace.inner()
        })
        .collect::<Vec<_>>();

    crate::traces_to_geojson(lines.first().unwrap(), lines.get(1).unwrap()).unwrap();

    Ok(ValidateReturn::Success(ValidateReturnSuccess::Success {
        success: true,
        data: (),
    }))
}
