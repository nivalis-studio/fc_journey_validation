use chrono::{DateTime, Utc};
use geo::{Coord, Point, Within};
use std::f64;

use crate::{france::FRANCE, input::PointInput, output::PointOutput};

#[derive(Debug, Clone)]
pub struct PointWithId {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub trace_id: String,
    pub timestamp: DateTime<Utc>,
}

impl PointWithId {
    pub fn is_in_france(&self) -> bool {
        Point::from(self).is_within(&*FRANCE)
    }

    pub fn get_ms_delta_with(&self, other: &PointWithId) -> i64 {
        self.timestamp
            .signed_duration_since(other.timestamp)
            .num_milliseconds()
            .abs()
    }
}

impl From<&PointInput> for PointWithId {
    fn from(value: &PointInput) -> Self {
        Self {
            trace_id: value.gps_trace_id.to_string(),
            timestamp: value.timestamp,
            id: value.id.to_string(),
            x: value.longitude,
            y: value.latitude,
        }
    }
}

impl From<&PointWithId> for Point<f64> {
    fn from(value: &PointWithId) -> Self {
        Self::from(Coord::from(value))
    }
}

impl From<&PointWithId> for Coord<f64> {
    fn from(value: &PointWithId) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<&PointWithId> for PointOutput {
    fn from(value: &PointWithId) -> Self {
        Self {
            id: value.id.to_owned(),
            timestamp: value.timestamp.to_owned(),
            longitude: value.x,
            latitude: value.y,
        }
    }
}
