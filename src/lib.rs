pub mod cli;
pub mod error;
pub mod journeys;
pub mod points;
pub mod traces;

mod traces_to_geojson;
mod validate_journey;

pub type Result<T, E = error::JourneyValidationError> = std::result::Result<T, E>;

pub use self::traces_to_geojson::traces_to_geojson;
pub use self::validate_journey::validate_journey;
