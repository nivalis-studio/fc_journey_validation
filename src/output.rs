use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::error::JourneyValidationError;

#[skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutputError {
    pub cancel_reason: String,
}

#[skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSuccess {
    pub common_distance: f64,
    pub common_start_point: PointOutput,
    pub common_end_point: PointOutput,
    pub average_confidence: f64,
    pub traces: TracesOutput,
}

#[allow(clippy::large_enum_variant)]
#[skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Output {
    Error(OutputError),
    Success(OutputSuccess),
    #[default]
    Empty,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TracesOutput {
    pub driver_trace: TraceOutput,
    pub passenger_trace: TraceOutput,
}

#[derive(Serialize)]
pub struct TraceOutput {
    pub id: String,
    pub points: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct PointOutput {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
}

impl From<JourneyValidationError> for Output {
    fn from(value: JourneyValidationError) -> Self {
        Self::Error(OutputError {
            cancel_reason: value.to_string(),
        })
    }
}
