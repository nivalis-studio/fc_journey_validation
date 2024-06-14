pub mod cli;
pub mod error;
pub mod journeys;
pub mod output;
pub mod points;
pub mod traces;

mod validate_journey;

pub type Result<T, E = error::JourneyValidationError> = std::result::Result<T, E>;

pub use self::validate_journey::validate_journey;
