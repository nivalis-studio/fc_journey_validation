use crate::{is_point_in_france, journeys::Journey};
use anyhow::Result;

const MAX_DELTA_IN_MILLISECONDS: u32 = 90_000;
const MIN_DISTANCE_IN_METERS: u16 = 1000;
const MAX_DISTANCE_IN_METERS: u32 = 80_000;

pub enum ValidateReturnError {
    Error {
        success: bool,
        reason: &'static str,
        readable_message: Option<&'static str>,
    },
}

pub enum ValidateReturnSuccess<T> {
    Success { success: bool, data: T },
}

pub enum ValidateReturn<T> {
    Error(ValidateReturnError),
    Success(ValidateReturnSuccess<T>),
}

pub fn validate_journey(journey: Option<Journey>) -> Result<ValidateReturn<()>> {
    if journey.is_none() {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Journey not found",
            readable_message: None,
        }));
    }

    let journey = journey.unwrap();

    if journey.start_time.is_none() {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Missing startTime",
            readable_message: None,
        }));
    }

    if journey.end_time.is_none() {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Missing endTime",
            readable_message: None,
        }));
    }

    if journey.driver_id == journey.passenger_id {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Driver cannot be passenger",
            readable_message: None,
        }));
    }

    let driver_trace = journey
        .gps_trace
        .iter()
        .find(|trace| trace.user_id == journey.driver_id);

    if driver_trace.is_none() {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Driver trace not found",
            readable_message: None,
        }));
    }

    let driver_trace = driver_trace.unwrap();

    if driver_trace.points.len() < 2 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Driver trace is empty",
            readable_message: None,
        }));
    }

    let passenger_trace = journey
        .gps_trace
        .iter()
        .find(|trace| trace.user_id == journey.passenger_id);

    if passenger_trace.is_none() {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Passenger trace not found",
            readable_message: None,
        }));
    }

    let passenger_trace = passenger_trace.unwrap();

    if passenger_trace.points.len() < 2 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Passenger trace is empty",
            readable_message: None,
        }));
    }

    let driver_start_point = driver_trace.points.get(0).unwrap();

    let passenger_start_point = passenger_trace.points.get(0).unwrap();

    let start_point_delta = driver_start_point
        .timestamp
        .signed_duration_since(passenger_start_point.timestamp)
        .num_seconds()
        .abs();

    if start_point_delta > MAX_DELTA_IN_MILLISECONDS as i64 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Start points timestamps are too far apart",
            readable_message: None,
        }));
    }

    let driver_end_point = driver_trace
        .points
        .get(driver_trace.points.len() - 1)
        .unwrap();

    let passenger_end_point = passenger_trace
        .points
        .get(passenger_trace.points.len() - 1)
        .unwrap();

    let end_point_delta = driver_end_point
        .timestamp
        .signed_duration_since(passenger_end_point.timestamp)
        .num_seconds()
        .abs();

    if end_point_delta > MAX_DELTA_IN_MILLISECONDS as i64 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "End points timestamps are too far apart",
            readable_message: None,
        }));
    }

    let is_in_france = is_point_in_france(driver_start_point).unwrap()
        || is_point_in_france(passenger_start_point).unwrap()
        || is_point_in_france(driver_end_point).unwrap()
        || is_point_in_france(passenger_end_point).unwrap();

    if !is_in_france {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "Not in France",
            readable_message: None,
        }));
    }

    let distance = 0.0;

    if distance < MIN_DISTANCE_IN_METERS as f64 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "distance too short",
            readable_message: None,
        }));
    }

    if distance > MAX_DISTANCE_IN_METERS as f64 {
        return Ok(ValidateReturn::Error(ValidateReturnError::Error {
            success: false,
            reason: "distance too long",
            readable_message: None,
        }));
    }

    Ok(ValidateReturn::Success(ValidateReturnSuccess::Success {
        success: true,
        data: (),
    }))
}