use serde::Serialize;

use crate::points::GpsPoint;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorOutput {
    success: bool,
    cancel_reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessOutput {
    distance_driver: f64,
    distance_passenger: f64,
    common_distance: f64,
    common_start_point: GpsPoint,
    common_end_point: GpsPoint,
    average_confidence: f32,
    traces: (TracesOuput, TracesOuput),
}

#[derive(Serialize)]
pub struct TracesOuput {
    id: String,
    points: Vec<GpsPoint>,
}
