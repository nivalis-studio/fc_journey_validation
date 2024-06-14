use crate::journeys::Journey;
use crate::Result;

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

    let traces = traces.validate()?.simplified();

    crate::traces_to_geojson(&traces.0, &traces.1).unwrap();

    Ok(ValidateReturn::Success(ValidateReturnSuccess::Success {
        success: true,
        data: (),
    }))
}
