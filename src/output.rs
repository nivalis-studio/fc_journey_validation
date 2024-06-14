use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{error::JourneyValidationError, points::GpsPoint};

#[skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub success: bool,
    cancel_reason: Option<String>,
    distance_driver: Option<f64>,
    distance_passenger: Option<f64>,
    common_distance: Option<f64>,
    common_start_point: Option<GpsPoint>,
    common_end_point: Option<GpsPoint>,
    average_confidence: Option<f32>,
    traces: Option<(TracesOuput, TracesOuput)>,
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
pub struct TracesOuput {
    id: String,
    points: Vec<GpsPoint>,
}
