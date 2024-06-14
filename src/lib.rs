pub mod cli;
pub mod error;
pub mod journeys;
pub mod output;
pub mod points;
pub mod traces;

pub type Result<T, E = error::JourneyValidationError> = std::result::Result<T, E>;
