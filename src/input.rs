use std::{
    io::{self, Read},
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{error::JourneyValidationError, Result};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JourneyInput {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub driver_id: Option<String>,
    pub passenger_id: Option<String>,
    pub gps_trace: Vec<TraceInput>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TraceInput {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: Option<String>,
    pub points: Vec<PointInput>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PointInput {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accuracy: Option<f64>,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f32>,
    pub speed: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub gps_trace_id: String,
}

impl JourneyInput {
    pub fn from_stdin() -> Result<Self> {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;

        JourneyInput::try_from(buffer.as_str())
    }
}

impl TryFrom<&str> for JourneyInput {
    type Error = JourneyValidationError;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        let journey: JourneyInput = serde_json::from_str(value)?;

        Ok(journey)
    }
}

impl TryFrom<PathBuf> for JourneyInput {
    type Error = JourneyValidationError;

    fn try_from(value: PathBuf) -> std::prelude::v1::Result<Self, Self::Error> {
        let json_string = std::fs::read_to_string(value)?;

        Self::try_from(json_string.as_ref())
    }
}
