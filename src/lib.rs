pub mod error;
pub mod france;
pub mod input;
pub mod journey;
pub mod point;
pub mod trace;

pub type Result<T, E = error::JourneyValidationError> = std::result::Result<T, E>;
