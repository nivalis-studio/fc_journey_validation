pub mod cli;
pub mod error;
pub mod france;
pub mod input;
pub mod journey;
pub mod output;
pub mod point;
pub mod trace;
pub mod visualize;

pub type Result<T, E = error::JourneyValidationError> = std::result::Result<T, E>;
