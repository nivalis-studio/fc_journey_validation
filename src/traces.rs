use std::collections::VecDeque;
use std::ops::Deref;

use chrono::{DateTime, Utc};
use geo::{
    coord, Closest, Coord, DensifyHaversine, FrechetDistance, HaversineClosestPoint,
    HaversineDistance, HaversineLength, OutlierDetection, RemoveRepeatedPoints, Simplify,
};
use geo::{LineString, Point};
use serde::{Deserialize, Serialize};

use crate::error::JourneyValidationError;
use crate::output::{Output, PointOutput, TraceOuput};
use crate::points::GpsPoint;
use crate::Result;

const MAX_DELTA_IN_MILLISECONDS: u32 = 90_000;
const MIN_DISTANCE_IN_METERS: u16 = 1000;
const MAX_DISTANCE_IN_METERS: u32 = 80_000;

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
        if self.points.len() < 2 {
            return Err(JourneyValidationError::EmptyTrace(self.id.to_owned()));
        }

        let start = self.points.first().unwrap();
        let end = self.points.last().unwrap();

        Ok((start, end))
    }

    pub fn is_in_france(&self) -> Result<bool> {
        let (start, end) = self.get_edges()?;

        Ok(start.is_in_france() || end.is_in_france())
    }
}

pub struct GpsTracesPair(pub GpsTrace, pub GpsTrace);

impl GpsTracesPair {
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

    pub fn get_confidence(&self) -> f64 {
        let (driver, passenger) = self.to_traces();

        let frechet_distance = driver.get_similarity(&passenger);

        1.0 - ((frechet_distance * 1000.0) / 100.0).clamp(0.0, 1.0)
    }

    pub fn get_result(&self) -> Result<Output> {
        let (driver_trace, passenger_trace) = self.to_traces();

        let confidence = self.get_confidence();

        let mut common_coords: Vec<Coord<f64>> = Vec::new();
        let mut common_points: VecDeque<GpsPoint> = VecDeque::new();

        let (points, other_trace) = match self.0.points.len() < self.1.points.len() {
            true => (&self.0.points, &passenger_trace),
            false => (&self.1.points, &driver_trace),
        };

        for driver_point in points.iter() {
            let point: Point<f64> = driver_point.into();
            let other_point: Closest<f64> = other_trace.haversine_closest_point(&point);

            let other_point: Point<f64> = match other_point {
                Closest::SinglePoint(point) => point,
                Closest::Intersection(intersection) => intersection,
                Closest::Indeterminate => continue,
            };

            let dist = point.haversine_distance(&other_point);

            if dist < 1000.0 {
                common_points.push_back(driver_point.clone());
                common_coords.push(Coord {
                    x: point.x(),
                    y: point.y(),
                })
            }
        }

        let common_line_string: LineString = LineString::new(common_coords);

        let common_start_point = common_points.pop_front().map(PointOutput::from);
        let common_end_point = common_points.pop_front().map(PointOutput::from);

        let distance = common_line_string.haversine_length();

        if distance < MIN_DISTANCE_IN_METERS as f64 {
            return Err(JourneyValidationError::InvalidDistance("short".into()));
        }

        if distance > MAX_DISTANCE_IN_METERS as f64 {
            return Err(JourneyValidationError::InvalidDistance("long".into()));
        }

        let traces: Option<(TraceOuput, TraceOuput)> = Some(((&self.0).into(), (&self.1).into()));

        let output = Output {
            success: true,
            average_confidence: Some(confidence),
            common_distance: Some(distance),
            distance_driver: Some(driver_trace.haversine_length()),
            distance_passenger: Some(passenger_trace.haversine_length()),
            common_start_point,
            common_end_point,
            traces,
            ..Default::default()
        };

        Ok(output)
    }

    pub fn to_traces(&self) -> (Trace, Trace) {
        let GpsTracesPair(driver, passenger) = self;

        (driver.into(), passenger.into())
    }

    pub fn to_simplified_traces(&self) -> (Trace, Trace) {
        let (driver, passenger) = self.to_traces();

        (driver.simplified(), passenger.simplified())
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

    pub fn get_similarity(&self, other: &Trace) -> f64 {
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
