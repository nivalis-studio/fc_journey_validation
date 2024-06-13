use chrono::{DateTime, Utc};
use geo::Point;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Journey {
    pub gps_trace: (GpsTrace, GpsTrace),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GpsTrace {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub distance: f64,
    pub hash: String,
    pub journey_id: String,
    pub user_id: String,
    pub points: Vec<GpsPoint>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GpsPoint {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accuracy: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub altitude_accuracy: f64,
    pub heading: f32,
    pub speed: f64,
    pub timestamp: DateTime<Utc>,
    pub gps_trace_id: String,
}

impl From<&GpsPoint> for Point {
    fn from(value: &GpsPoint) -> Self {
        Self::new(value.latitude, value.longitude)
    }
}
