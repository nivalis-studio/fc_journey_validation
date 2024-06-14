use crate::traces::Trace;
use crate::Result;
use crate::{journeys::Journey, validate_traces};
use geo::{Closest, Coord, HaversineLength, LineString, Point};
use geo::{HaversineClosestPoint, HaversineDistance};

const MAX_DELTA_IN_MILLISECONDS: u32 = 90_000;
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
    let (driver_trace, passenger_trace) = journey.validate()?;

    let driver_start_point = driver_trace.points.first().unwrap();

    let passenger_start_point = passenger_trace.points.first().unwrap();

    let start_point_delta = driver_start_point
        .timestamp
        .signed_duration_since(passenger_start_point.timestamp)
        .num_seconds()
        .abs();

    if start_point_delta > MAX_DELTA_IN_MILLISECONDS as i64 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Start points timestamps are too far apart",
        }));
    }

    let driver_end_point = driver_trace.points.last().unwrap();

    let passenger_end_point = passenger_trace.points.last().unwrap();

    let end_point_delta = driver_end_point
        .timestamp
        .signed_duration_since(passenger_end_point.timestamp)
        .num_seconds()
        .abs();

    if end_point_delta > MAX_DELTA_IN_MILLISECONDS as i64 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "End points timestamps are too far apart",
        }));
    }

    let is_in_france = driver_start_point.is_in_france()
        || passenger_start_point.is_in_france()
        || driver_end_point.is_in_france()
        || passenger_end_point.is_in_france();

    if !is_in_france {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Not in France",
        }));
    }

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
