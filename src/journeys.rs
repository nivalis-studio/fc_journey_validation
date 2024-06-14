use std::{
    io::{self, Read},
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    error::JourneyValidationError,
    output::Output,
    traces::{GpsTrace, GpsTracesPair},
    Result,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Journey {
    pub index: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub start_city: Option<String>,
    pub end_city: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub flagged: bool,
    pub status: String,
    pub processed: bool,
    pub average_confidence: Option<f64>,
    pub tolerance: Option<f64>,
    pub distance: Option<f64>,
    pub driver_id: Option<String>,
    pub passenger_id: Option<String>,
    pub cancel_reason: Option<String>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub gps_trace: Vec<GpsTrace>,
}

impl Journey {
    pub fn from_stdin() -> Result<Self> {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;

        Journey::try_from(buffer)
    }
    pub fn validate(self) -> Result<Output> {
        let traces = self.get_traces()?.validate()?;

        traces.get_result()
    }

    pub fn get_traces(&self) -> Result<GpsTracesPair> {
        let driver_id = self
            .driver_id
            .as_ref()
            .ok_or(JourneyValidationError::MissingDriver)?;

        let passenger_id = self
            .passenger_id
            .as_ref()
            .ok_or(JourneyValidationError::MissingPassenger)?;

        if passenger_id == driver_id {
            return Err(JourneyValidationError::InvalidPassenger);
        }

        let driver_trace = self
            .get_user_trace(driver_id)
            .ok_or(JourneyValidationError::MissingTrace("driver".into()))?;

        let passenger_trace = self
            .get_user_trace(passenger_id)
            .ok_or(JourneyValidationError::MissingTrace("passenger".into()))?;

        Ok(GpsTracesPair(driver_trace, passenger_trace))
    }

    pub fn get_user_trace(&self, user_id: &str) -> Option<GpsTrace> {
        self.gps_trace
            .iter()
            .find(|t| t.user_id == user_id)
            .cloned()
    }
}

impl TryFrom<&str> for Journey {
    type Error = JourneyValidationError;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        let journey: Journey = serde_json::from_str(value)?;

        if journey.start_time.is_none() {
            return Err(JourneyValidationError::MissingStartTime);
        }
        if journey.end_time.is_none() {
            return Err(JourneyValidationError::MissingEndTime);
        }

        Ok(journey)
    }
}

impl TryFrom<String> for Journey {
    type Error = JourneyValidationError;

    fn try_from(value: String) -> std::prelude::v1::Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<PathBuf> for Journey {
    type Error = JourneyValidationError;

    fn try_from(value: PathBuf) -> std::prelude::v1::Result<Self, Self::Error> {
        let json_string = std::fs::read_to_string(value)?;

        Self::try_from(json_string.as_ref())
    }
}
