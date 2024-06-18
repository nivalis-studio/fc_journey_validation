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
            id: value.id.clone(),
            trace_id: value.gps_trace_id.clone(),
            timestamp: value.timestamp,
            x: value.longitude,
            y: value.latitude,
        }
    }
}

impl From<&PointWithId> for Point<f64> {
    fn from(value: &PointWithId) -> Self {
        Point::new(value.x, value.y)
    }
}

impl From<&PointWithId> for Coord<f64> {
    fn from(value: &PointWithId) -> Self {
        Coord {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<&PointWithId> for PointOutput {
    fn from(value: &PointWithId) -> Self {
        Self {
            id: value.id.clone(),
            timestamp: value.timestamp,
            longitude: value.x,
            latitude: value.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_is_in_france() {
        let point_in_france = PointWithId {
            id: "1".to_string(),
            x: 2.3522,
            y: 48.8566,
            trace_id: "trace_1".to_string(),
            timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 0, 0).unwrap(),
        };

        assert!(point_in_france.is_in_france());

        let point_not_in_france = PointWithId {
            id: "1".to_string(),
            x: 0.0,
            y: 0.0,
            trace_id: "trace_1".to_string(),
            timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 0, 0).unwrap(),
        };

        assert!(!point_not_in_france.is_in_france());
    }

    #[test]
    fn test_get_ms_delta_with() {
        let point1 = PointWithId {
            id: "1".to_string(),
            x: 2.3522,
            y: 48.8566,
            trace_id: "trace_1".to_string(),
            timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 0, 0).unwrap(),
        };

        let point2 = PointWithId {
            id: "2".to_string(),
            x: 2.3522,
            y: 48.8566,
            trace_id: "trace_2".to_string(),
            timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 0, 1).unwrap(),
        };

        assert_eq!(point1.get_ms_delta_with(&point2), 1000);
    }
}
