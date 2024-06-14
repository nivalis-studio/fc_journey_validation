use std::ops::Deref;

use chrono::{DateTime, Utc};
use geo::{
    coord, DensifyHaversine, FrechetDistance, OutlierDetection, RemoveRepeatedPoints, Simplify,
};
use geo::{LineString, Point};
use serde::{Deserialize, Serialize};

use crate::error::JourneyValidationError;
use crate::points::GpsPoint;
use crate::Result;

const MAX_DELTA_IN_MILLISECONDS: u32 = 90_000;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GpsTrace {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub distance: Option<f64>,
    pub hash: Option<String>,
    pub journey_id: String,
    pub user_id: String,
    pub points: Vec<GpsPoint>,
}

impl GpsTrace {
    pub fn validate(self) -> Result<Self> {
        if self.points.len() < 2 {
            return Err(JourneyValidationError::EmptyTrace(self.id.to_owned()));
        }

        Ok(self)
    }

    pub fn get_edges(&self) -> Result<(&GpsPoint, &GpsPoint)> {
        let start = self
            .points
            .first()
            .ok_or(JourneyValidationError::EmptyTrace(self.id.to_owned()))?;
        let end = self
            .points
            .last()
            .ok_or(JourneyValidationError::EmptyTrace(self.id.to_owned()))?;

        Ok((start, end))
    }

    pub fn is_in_france(&self) -> Result<bool> {
        let (start, end) = self.get_edges()?;

        Ok(start.is_in_france() || end.is_in_france())
    }
}

pub struct TracesPair(pub GpsTrace, pub GpsTrace);

impl TracesPair {
    pub fn validate(self) -> Result<Self> {
        let driver_trace = &self.0;
        let passenger_trace = &self.1;
        let (driver_start, driver_end) = driver_trace.get_edges()?;
        let (passenger_start, passenger_end) = passenger_trace.get_edges()?;

        let validate_timestamps_delta = |points: (&GpsPoint, &GpsPoint), name: &str| {
            let first = points.0;
            let second = points.1;

            let delta = first
                .timestamp
                .signed_duration_since(second.timestamp)
                .num_seconds()
                .abs();

            if delta > MAX_DELTA_IN_MILLISECONDS as i64 {
                return Err(JourneyValidationError::TimestampsDeltaTooBig(name.into()));
            }

            Ok(())
        };

        validate_timestamps_delta((driver_start, passenger_start), "start")?;
        validate_timestamps_delta((driver_end, passenger_end), "end")?;

        if !driver_trace.is_in_france()? || !passenger_trace.is_in_france()? {
            return Err(JourneyValidationError::NotInFrance);
        }

        Ok(self)
    }
}

#[derive(Clone, Debug)]
pub struct Trace(LineString);

impl Trace {
    pub fn inner(self) -> LineString {
        self.0
    }

    pub fn simplified(&self) -> Self {
        let line_string = self.remove_repeated_points().simplify(&0.00001);

        Self(line_string)
    }

    pub fn compare_distance(&self, other: &Trace) -> f64 {
        self.frechet_distance(other)
    }
}

impl AsRef<LineString> for Trace {
    fn as_ref(&self) -> &LineString {
        &self.0
    }
}

impl Deref for Trace {
    type Target = LineString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&GpsTrace> for Trace {
    fn from(value: &GpsTrace) -> Self {
        let coords: Vec<Point> = value.points.iter().map(Point::from).collect();

        let line_string: LineString = coords
            .iter()
            .zip(coords.outliers(3).iter())
            .filter(|(_, &score)| score <= 1.0) // Adjust threshold as needed
            .map(|(&point, _)| coord! { x: point.x(), y: point.y() })
            .collect();

        let line_string = line_string.densify_haversine(10.0);
        Self(line_string)
    }
}

impl From<GpsTrace> for Trace {
    fn from(value: GpsTrace) -> Self {
        Self::from(&value)
    }
}
