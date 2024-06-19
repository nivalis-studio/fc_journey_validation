use std::{collections::HashMap, f64, marker::PhantomData};

use chrono::{DateTime, Utc};
use geo::{
    FrechetDistance, HaversineBearing, HaversineDistance, HaversineLength, LineString, Point,
    RemoveRepeatedPoints, Simplify,
};

use crate::{
    error::JourneyValidationError,
    input::TraceInput,
    output::{PointOutput, TraceOutput},
    point::PointWithId,
    visualize::{visualize, FeatureProperties},
    Result,
};

const MAX_POINTS_DELTA_IN_METERS: f64 = 1000.0;
const MAX_BEARING: f64 = 90.0;

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

        all_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let tx = all_points.last().unwrap();
        let trace_without_tx = if tx.trace_id == self.id { self } else { other };
        let ty_data = all_points.iter().enumerate().rfind(|(_, p)| {
            if p.trace_id == tx.trace_id {
                return false;
            }

            let point: Point<f64> = Point::from(**p);

            trace_without_tx
                .points
                .iter()
                .rev()
                .any(|p| Point::from(p).haversine_distance(&point) < MAX_POINTS_DELTA_IN_METERS)
        });

        let (ty_idx, ty) = match ty_data {
            Some(info) => info,
            None => return Err(JourneyValidationError::NoCommonPoints),
        };

        let all_points = &all_points[0..=ty_idx];

        let mut common_points: Vec<&PointWithId> = Vec::new();

        for (idx, curr) in all_points.iter().enumerate() {
            let prev = if idx > 0 {
                all_points.get(idx - 1)
            } else {
                None
            };
            let next = all_points.get(idx + 1);

            if let (Some(prev), Some(next)) = (prev, next) {
                let point = Point::from(*curr);
                let prev_point = Point::from(*prev);
                let next_point = Point::from(*next);

                let bearing_prev = point.haversine_bearing(prev_point);
                let bearing_next = point.haversine_bearing(next_point);

                if bearing_prev < MAX_BEARING && bearing_next < MAX_BEARING {
                    continue;
                }
            }
            common_points.push(curr)
        }

        let common_linestring = LineString::from(common_points).simplify(&0.00001);

        let common_distance = common_linestring.haversine_length();

        visualize([(common_linestring, FeatureProperties::new().color("#00ff00"))]);

        Ok(CommonTrace {
            common_distance,
            common_start_point: PointOutput::from(*all_points.first().unwrap()),
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
                    x: 2.295,
                    y: 48.8738,
                    trace_id: "trace_1".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 10, 0).unwrap(),
                },
                PointWithId {
                    id: "3".to_string(),
                    x: 2.3333,
                    y: 48.8606,
                    trace_id: "trace_1".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 20, 0).unwrap(),
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
                    x: 2.296,
                    y: 48.875,
                    trace_id: "trace_2".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 10, 0).unwrap(),
                },
                PointWithId {
                    id: "6".to_string(),
                    x: 2.3333,
                    y: 48.8606,
                    trace_id: "trace_2".to_string(),
                    timestamp: Utc.with_ymd_and_hms(2024, 6, 18, 12, 20, 0).unwrap(),
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
        assert_eq!(length, 7763.121089616901);
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

        assert_eq!(common_trace.common_distance, 7916.0507217867325);
        assert_eq!(common_trace.common_start_point.id, "1");
        assert_eq!(common_trace.common_end_point.id, "3");
    }
}
