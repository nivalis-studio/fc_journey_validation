use std::collections::HashMap;
use std::ops::Deref;

use anyhow::Context;
use chrono::{DateTime, Utc};
use geo::{
    coord, Closest, Coord, DensifyHaversine, FrechetDistance, HaversineClosestPoint,
    HaversineDistance, HaversineLength, OutlierDetection, RemoveRepeatedPoints, Simplify,
};
use geo::{LineString, Point};
use geojson::{Feature, FeatureCollection, Geometry, JsonObject, JsonValue};
use serde::{Deserialize, Serialize};

use crate::error::JourneyValidationError;
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

        self.validate_distance()?;

        Ok(self)
    }

    pub fn validate_distance(&self) -> Result<&Self> {
        let driver_trace = Trace::from(&self.0);
        let passenger_trace = Trace::from(&self.1);

        let confidence = driver_trace.get_confidence(&passenger_trace);

        let mut common_coords: Vec<Coord<f64>> = Vec::new();

        for driver_point in self.0.points.iter() {
            let driver_point: Point<f64> = driver_point.into();
            let passenger_point: Closest<f64> =
                passenger_trace.haversine_closest_point(&driver_point);

            let passenger_point: Point<f64> = match passenger_point {
                Closest::SinglePoint(point) => point,
                Closest::Intersection(intersection) => intersection,
                Closest::Indeterminate => continue,
            };

            let dist = driver_point.haversine_distance(&passenger_point);

            if dist < 1000.0 {
                common_coords.push(Coord {
                    x: driver_point.x(),
                    y: driver_point.y(),
                })
            }
        }

        let common_line_string: LineString = LineString::new(common_coords);

        let start_point = common_line_string.0.first();
        let end_point = common_line_string.0.last();

        let distance = common_line_string.haversine_length();

        if distance < MIN_DISTANCE_IN_METERS as f64 {
            return Err(JourneyValidationError::InvalidDistance("short".into()));
        }

        if distance > MAX_DISTANCE_IN_METERS as f64 {
            return Err(JourneyValidationError::InvalidDistance("long".into()));
        }

        Ok(self)
    }

    pub fn simplified(&self) -> (Trace, Trace) {
        let driver_trace = Trace::from(&self.0).simplified();
        let passenger_trace = Trace::from(&self.1).simplified();

        (driver_trace, passenger_trace)
    }

    pub fn to_geojson(&self) -> FeatureCollection {
        let (driver_trace, passenger_trace) = self.simplified();

        let create_properties = |color: &str, width: &str, opacity: &str| -> Option<JsonObject> {
            let mut properties = JsonObject::new();
            let properties_: HashMap<String, JsonValue> = [
                ("stroke".to_string(), JsonValue::from(color)),
                ("stroke-width".to_string(), JsonValue::from(width)),
                ("stroke-opacity".to_string(), JsonValue::from(opacity)),
            ]
            .iter()
            .cloned()
            .collect();

            properties.extend(properties_);

            Some(properties)
        };

        FeatureCollection {
            bbox: None,
            features: vec![
                Feature {
                    bbox: None,
                    geometry: Some(Geometry {
                        value: geojson::Value::from(&*driver_trace),
                        bbox: None,
                        foreign_members: None,
                    }),
                    id: None,
                    properties: create_properties("#00a3d7", "2", "1"),
                    foreign_members: None,
                },
                Feature {
                    bbox: None,
                    geometry: Some(Geometry {
                        value: geojson::Value::from(&*passenger_trace),
                        bbox: None,
                        foreign_members: None,
                    }),
                    id: None,
                    properties: create_properties("#ff6251", "2", "1"),
                    foreign_members: None,
                },
            ],
            foreign_members: None,
        }
    }

    pub fn visualize(&self) -> anyhow::Result<()> {
        let geojson = self.to_geojson().to_string();

        let uri_data = urlencoding::encode(&geojson);
        let url = format!("http://geojson.io/#data=data:application/json,{}", uri_data);

        open::that(url).context("Failed to open geojson in the default browser")?;

        Ok(())
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

    pub fn get_confidence(&self, other: &Trace) -> f64 {
        let frechet_distance = self.frechet_distance(other);

        1.0 - ((frechet_distance * 1000.0) / 100.0).clamp(0.0, 1.0)
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
