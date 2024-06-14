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
    let traces = journey.get_traces()?;

    let traces = traces.validate()?;

    traces.visualize()?;

    traces.simplified();
    Ok(ValidateReturn::Success(ValidateReturnSuccess::Success {
        success: true,
        data: (),
    }))
}
