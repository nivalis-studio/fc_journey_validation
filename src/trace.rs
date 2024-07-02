use std::{collections::HashMap, f64, marker::PhantomData};

use chrono::{DateTime, Utc};
use geo::{
    EuclideanDistance, FrechetDistance, Geometry, HaversineBearing, HaversineDistance,
    HaversineLength, LineString, Point, RemoveRepeatedPoints, Simplify,
};

use crate::{
    error::JourneyValidationError,
    input::TraceInput,
    output::{PointOutput, TraceOutput},
    point::PointWithId,
    // visualize::{visualize, FeatureProperties},
    Result,
};

const MAX_POINTS_DELTA_IN_METERS: f64 = 1000.0;
const MAX_BEARING: f64 = 50.0;

pub struct Simplified;
pub struct NotSimplified;

#[derive(Debug)]
pub struct Trace<T = NotSimplified> {
    pub id: String,
    pub points: Vec<PointWithId>,
    status: PhantomData<T>,
}

impl Trace {
    pub fn from_linestring<T>(&self, linestring: LineString<f64>) -> Trace<T> {
        let coords: HashMap<(u64, u64), (String, DateTime<Utc>)> = self
            .points
            .iter()
            .map(|p| {
                (
                    (p.x.to_bits(), p.y.to_bits()),
                    (p.id.to_string(), p.timestamp),
                )
            })
            .collect();

        let points = linestring
            .into_iter()
            .filter_map(|coord| {
                coords
                    .get(&(coord.x.to_bits(), coord.y.to_bits()))
                    .map(|(id, timestamp)| PointWithId {
                        trace_id: self.id.clone(),
                        id: id.clone(),
                        timestamp: timestamp.to_owned(),
                        x: coord.x,
                        y: coord.y,
                    })
            })
            .collect();

        Trace {
            id: self.id.clone(),
            points,
            status: PhantomData,
        }
    }

    pub fn get_edges(&self) -> (&PointWithId, &PointWithId) {
        let start_point = self.points.first().unwrap();
        let end_point = self.points.last().unwrap();

        (start_point, end_point)
    }

    pub fn common_trace_with(&self, other: &Trace) -> Result<CommonTrace> {
        let mut all_points: Vec<&PointWithId> =
            self.points.iter().chain(other.points.iter()).collect();
        all_points.sort_by_key(|p| p.timestamp);

        let t0 = all_points.first().unwrap();
        let tx = all_points.last().unwrap();
        let trace_with_tx = if tx.trace_id == self.id { self } else { other };
        let ls_with_tx = Geometry::LineString(LineString::from(trace_with_tx));

        let ty_data = all_points.iter().enumerate().rfind(|(_, p)| {
            if p.trace_id == tx.trace_id {
                return false;
            }

            Geometry::Point(Point::from(**p)).euclidean_distance(&ls_with_tx)
                < MAX_POINTS_DELTA_IN_METERS
        });

        let (ty_idx, ty) = match ty_data {
            Some(data) => data,
            None => return Err(JourneyValidationError::NoCommonPoints),
        };

        let all_points = &all_points[0..=ty_idx];

        let mut common_points: Vec<&PointWithId> = Vec::with_capacity(all_points.len());
        let mut idx = 0;

        while idx < all_points.len() {
            let prev = common_points.last();
            let mid = all_points[idx];
            let next = all_points.get(idx + 1);

            idx += 1;

            if idx == 1 {
                common_points.push(mid);
                continue;
            }

            if mid.id == ty.id {
                common_points.push(mid);
                break;
            }

            if prev.is_none() || next.is_none() {
                common_points.push(mid);
                continue;
            }

            let prev = prev.unwrap();
            let next = next.unwrap();

            if prev.trace_id == mid.trace_id && next.trace_id == mid.trace_id {
                common_points.push(mid);
                continue;
            }

            let prev_point = Point::from(*prev);
            let next_point = Point::from(*next);

            let prev_next_dist = prev_point.haversine_distance(&next_point);

            if prev.trace_id != mid.trace_id
                && prev.trace_id == next.trace_id
                && prev_next_dist < 250.0
            {
                common_points.push(next);
                idx += 1;
                continue;
            }

            let mid_point = Point::from(mid);
            let bearing_prev = mid_point.haversine_bearing(prev_point);
            let bearing_next = mid_point.haversine_bearing(next_point);
            let delta = (bearing_next - bearing_prev + 360.0) % 360.0;
            let angle = if delta <= 180.0 { delta } else { 360.0 - delta };

            if angle >= MAX_BEARING {
                common_points.push(mid);
                continue;
            }
        }

        common_points.sort_by_key(|p| p.timestamp);
        let mut filtered_points: Vec<&PointWithId> = Vec::with_capacity(common_points.len());

        let window_size = 5;
        let max = window_size - 1;

        if common_points.len() <= window_size {
            filtered_points.clone_from(&common_points);
        }

        for window in common_points.windows(window_size) {
            if window[0].id == t0.id {
                filtered_points.push(window[0]);
                continue;
            }

            if window[max].id == ty.id {
                filtered_points.push(window[max]);
            }

            if filtered_points.iter().any(|p| p.id == window[0].id) {
                continue;
            }

            let start_point = Point::from(window[0]);
            let end_point = Point::from(window[max]);
            let size = start_point.haversine_distance(&end_point);

            if size > 100.0 {
                filtered_points.push(window[0]);
                continue;
            }

            let trace_ids_count = window
                .iter()
                .map(|p| p.trace_id.clone())
                .collect::<Vec<String>>()
                .iter()
                .fold(HashMap::new(), |mut acc, id| {
                    *acc.entry(id.clone()).or_insert(0) += 1;
                    acc
                });

            if trace_ids_count.values().any(|&v| v >= max - 1) {
                for &point in window {
                    if trace_ids_count[&point.trace_id] >= max - 1 {
                        filtered_points.push(point);
                    }
                }
            }

            filtered_points.push(window[0]);
        }

        filtered_points.sort_by_key(|p| p.timestamp);
        common_points = filtered_points;
        filtered_points = Vec::with_capacity(common_points.len());

        let window_size = 2;
        let max = window_size - 1;

        for window in common_points.windows(window_size) {
            if filtered_points.iter().any(|p| p.id == window[0].id) {
                continue;
            }

            if window[0].id == t0.id {
                filtered_points.push(window[0]);
            }

            if window[max].id == ty.id {
                filtered_points.push(window[max]);
            }

            let prev = filtered_points.last();

            if prev.is_none() {
                continue;
            }

            let prev = prev.unwrap();

            let prev_point = Point::from(*prev);
            let mid_point = Point::from(window[0]);
            let next_point = Point::from(window[max]);
            let bearing_prev = mid_point.haversine_bearing(prev_point);
            let bearing_next = mid_point.haversine_bearing(next_point);
            let delta = (bearing_next - bearing_prev + 360.0) % 360.0;
            let angle = if delta <= 180.0 { delta } else { 360.0 - delta };

            if angle == 0.0 || angle >= MAX_BEARING {
                filtered_points.push(window[0]);
            }
        }

        filtered_points.sort_by_key(|p| p.timestamp);
        let common_linestring = LineString::from(filtered_points).simplify(&0.00001);
        let common_distance = common_linestring.haversine_length();

        // visualize([
        //     (common_linestring, FeatureProperties::new().color("#00ffff")),
        //     (
        //         LineString::from(self),
        //         FeatureProperties::new().color("#ff0000"),
        //     ),
        //     (
        //         LineString::from(other),
        //         FeatureProperties::new().color("#00ff00"),
        //     ),
        // ]);

        Ok(CommonTrace {
            common_distance,
            common_start_point: PointOutput::from(*t0),
            common_end_point: PointOutput::from(*ty),
        })
    }
}

pub struct CommonTrace {
    pub common_distance: f64,
    pub common_start_point: PointOutput,
    pub common_end_point: PointOutput,
}

impl Trace<NotSimplified> {
    pub fn simplified(&self, epsilon: f64) -> Trace<Simplified> {
        self.from_linestring(
            LineString::from(self)
                .remove_repeated_points()
                .simplify(&epsilon),
        )
    }
}

impl Trace<Simplified> {
    pub fn confidence_with(&self, other: &Trace<Simplified>) -> f64 {
        1.0 - ((LineString::from(self).frechet_distance(&other.into()) * 1000.0) / 100.0)
            .clamp(0.0, 1.0)
    }
}

impl<T> Trace<T> {
    pub fn haversine_length(&self) -> f64 {
        LineString::from(self).haversine_length()
    }
}

impl From<Trace<Simplified>> for TraceOutput {
    fn from(value: Trace<Simplified>) -> Self {
        Self {
            id: value.id.clone(),
            distance: value.haversine_length(),
            points: value.points.into_iter().map(|p| p.id).collect(),
        }
    }
}

impl<T> From<&Trace<T>> for LineString<f64> {
    fn from(value: &Trace<T>) -> Self {
        LineString::from(
            value
                .points
                .iter()
                .map(Point::from)
                .collect::<Vec<Point<f64>>>(),
        )
    }
}

impl From<&TraceInput> for Trace {
    fn from(value: &TraceInput) -> Self {
        let points = value.points.iter().map(PointWithId::from).collect();

        Self {
            id: value.id.clone(),
            points,
            status: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_sample_points() -> (Vec<PointWithId>, Vec<PointWithId>) {
        (
            vec![
                PointWithId {
                    id: "1".to_string(),
                    x: 2.3522,
                    y: 48.8566,
                    trace_id: "trace_1".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 0, 0).unwrap(),
                },
                PointWithId {
                    id: "2".to_string(),
                    x: 2.3333,
                    y: 48.8606,
                    trace_id: "trace_1".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 20, 2).unwrap(),
                },
                PointWithId {
                    id: "3".to_string(),
                    x: 2.295,
                    y: 48.8738,
                    trace_id: "trace_1".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 20, 4).unwrap(),
                },
            ],
            vec![
                PointWithId {
                    id: "4".to_string(),
                    x: 2.3522,
                    y: 48.8566,
                    trace_id: "trace_2".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 0, 0).unwrap(),
                },
                PointWithId {
                    id: "5".to_string(),
                    x: 2.3333,
                    y: 48.8606,
                    trace_id: "trace_2".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 20, 2).unwrap(),
                },
                PointWithId {
                    id: "6".to_string(),
                    x: 2.296,
                    y: 48.875,
                    trace_id: "trace_2".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 20, 4).unwrap(),
                },
            ],
        )
    }

    #[test]
    fn test_get_edges() {
        let (points, _) = create_sample_points();
        let trace = Trace {
            id: "trace_1".to_string(),
            points,
            status: PhantomData::<NotSimplified>,
        };

        let (start, end) = trace.get_edges();
        assert_eq!(start.id, "1");
        assert_eq!(end.id, "3");
    }

    #[test]
    fn test_haversine_length() {
        let (points, _) = create_sample_points();
        let trace = Trace {
            id: "trace_1".to_string(),
            points,
            status: PhantomData::<NotSimplified>,
        };

        let length = trace.haversine_length();
        assert_eq!(length, 4615.121822251224);
    }

    #[test]
    fn test_confidence_with() {
        let (points1, points2) = create_sample_points();
        let trace1 = Trace {
            id: "trace_1".to_string(),
            points: points1,
            status: PhantomData::<Simplified>,
        };

        let trace2 = Trace {
            id: "trace_2".to_string(),
            points: points2,
            status: PhantomData::<Simplified>,
        };

        let distance = trace1.confidence_with(&trace2);
        assert_eq!(distance, 0.9843795006482089);
    }

    #[test]
    fn test_common_trace_with() {
        let (points1, points2) = create_sample_points();
        let trace1 = Trace {
            id: "trace_1".to_string(),
            points: points1,
            status: PhantomData::<NotSimplified>,
        };

        let trace2 = Trace {
            id: "trace_2".to_string(),
            points: points2,
            status: PhantomData::<NotSimplified>,
        };

        let common_trace = trace1.common_trace_with(&trace2).unwrap();

        assert_eq!(common_trace.common_distance, 4615.121822251224);
        assert_eq!(common_trace.common_start_point.id, "1");
        assert_eq!(common_trace.common_end_point.id, "3");
    }
}
