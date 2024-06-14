use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{error::JourneyValidationError, traces::GpsTrace, Result};

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
    pub fn validate(&self) -> Result<()> {
        if !self.has_startime() {
            return Err(JourneyValidationError::MissingStartTime);
        }
        if !self.has_endtime() {
            return Err(JourneyValidationError::MissingEndTime);
        }
        if !self.has_driver() {
            return Err(JourneyValidationError::MissingDriver);
        }
        if !self.has_passenger() {
            return Err(JourneyValidationError::MissingPassenger);
        }
        if !self.has_valid_passenger() {
            return Err(JourneyValidationError::InvalidPassenger);
        }

        Ok(())
    }

    pub fn has_startime(&self) -> bool {
        self.start_time.is_some()
    }

    pub fn has_endtime(&self) -> bool {
        self.end_time.is_some()
    }

    pub fn has_driver(&self) -> bool {
        self.driver_id.is_some()
    }

    pub fn has_passenger(&self) -> bool {
        self.passenger_id.is_some()
    }

    pub fn has_valid_passenger(&self) -> bool {
        self.has_passenger() && self.passenger_id != self.driver_id
    }
}

impl TryFrom<&str> for Journey {
    type Error = JourneyValidationError;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        Ok(serde_json::from_str(value)?)
    }
}

impl TryFrom<PathBuf> for Journey {
    type Error = JourneyValidationError;

    fn try_from(value: PathBuf) -> std::prelude::v1::Result<Self, Self::Error> {
        let json_string = std::fs::read_to_string(value)?;

        Self::try_from(json_string.as_ref())
    }
}
