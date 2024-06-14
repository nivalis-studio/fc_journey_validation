use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{error::JourneyValidationError, points::GpsPoint};

#[skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub success: bool,
    pub cancel_reason: Option<String>,
    pub distance_driver: Option<f64>,
    pub distance_passenger: Option<f64>,
    pub common_distance: Option<f64>,
    pub common_start_point: Option<GpsPoint>,
    pub common_end_point: Option<GpsPoint>,
    pub average_confidence: Option<f64>,
    pub traces: Option<(TraceOuput, TraceOuput)>,
}

impl Output {
    pub fn success() -> Self {
        Self {
            success: true,
            ..Default::default()
        }
    }
}

impl From<JourneyValidationError> for Output {
    fn from(value: JourneyValidationError) -> Self {
        Self {
            success: false,
            cancel_reason: Some(value.to_string()),
            ..Default::default()
        }
    }
}

#[derive(Serialize)]
pub struct TraceOuput {
    pub id: String,
    pub points: Vec<GpsPoint>,
}
