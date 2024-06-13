use chrono::{DateTime, Utc};
use geo::Point;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Journey {
    pub index: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub start_city: Option<String>,
    pub end_city: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub flagged: bool,
    pub status: String,
    pub processed: bool,
    pub average_confidence: f64,
    pub tolerance: f64,
    pub distance: f64,
    pub driver_id: String,
    pub passenger_id: String,
    pub cancel_reason: Option<String>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub gps_trace: (GpsTrace, GpsTrace),
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GpsPoint {
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

impl From<&GpsPoint> for Point {
    fn from(value: &GpsPoint) -> Self {
        Self::new(value.longitude, value.latitude)
    }
}
