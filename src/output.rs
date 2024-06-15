use chrono::{DateTime, Utc};
use geo::Coord;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    error::JourneyValidationError,
    points::GpsPoint,
    traces::{GpsTrace, Trace},
};

#[skip_serializing_none]
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub success: bool,
    pub cancel_reason: Option<String>,
    pub distance_driver: Option<f64>,
    pub distance_passenger: Option<f64>,
    pub common_distance: Option<f64>,
    pub common_start_point: Option<PointOutput>,
    pub common_end_point: Option<PointOutput>,
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
    pub points: Vec<String>,
}

impl From<&GpsTrace> for TraceOuput {
    fn from(value: &GpsTrace) -> Self {
        let line_string = Trace::from(value).simplified().inner();

        let coords: Vec<Coord> = line_string.into_iter().collect();

        let points = value
            .points
            .iter()
            .filter(|gp| {
                coords
                    .iter()
                    .any(|c| c.x == gp.longitude && c.y == gp.latitude)
            })
            .map(|gp| gp.id.to_owned())
            .collect();

        Self {
            id: value.id.to_owned(),
            points,
        }
    }
}

#[derive(Serialize)]
pub struct PointOutput {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
}

impl From<GpsPoint> for PointOutput {
    fn from(value: GpsPoint) -> Self {
        Self {
            id: value.id,
            timestamp: value.timestamp,
            latitude: value.latitude,
            longitude: value.longitude,
        }
    }
}
