use std::{collections::HashMap, f64, marker::PhantomData};

use chrono::{DateTime, Utc};
use geo::{
    FrechetDistance, HaversineBearing, HaversineDistance, HaversineLength, LineString, Point,
    RemoveRepeatedPoints, Simplify,
};

use crate::{
    input::TraceInput,
    output::{PointOutput, TraceOutput},
    point::PointWithId,
};

const MAX_POINTS_DELTA_IN_METERS: u16 = 1000;
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
                        trace_id: self.id.to_string(),
                        id: id.to_string(),
                        timestamp: timestamp.to_owned(),
                        x: coord.x,
                        y: coord.y,
                    })
            })
            .collect();

        Trace {
            id: self.id.to_string(),
            points,
            status: PhantomData,
        }
    }

    pub fn get_edges(&self) -> (&PointWithId, &PointWithId) {
        let start_point = self.points.first().unwrap();
        let end_point = self.points.last().unwrap();

        (start_point, end_point)
    }

    pub fn haversine_length(&self) -> f64 {
        let linestring = LineString::from(self);
        linestring.haversine_length()
    }

    pub fn frechet_distance_with(&self, other: &Trace) -> f64 {
        LineString::from(self).frechet_distance(&other.into())
    }

    pub fn common_trace_with(&self, other: &Trace) -> CommonTrace {
        let mut all_points: Vec<&PointWithId> =
            self.points.iter().chain(other.points.iter()).collect();

        all_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let tx = all_points.last().unwrap();
        let trace_without_tx = if tx.trace_id == self.id { self } else { other };
        let (ty_idx, ty) = all_points
            .iter()
            .enumerate()
            .rfind(|(_, p)| {
                if p.trace_id == tx.trace_id {
                    return false;
                }

                let point: Point<f64> = Point::from(**p);

                trace_without_tx.points.iter().rev().any(|p| {
                    Point::from(p).haversine_distance(&point) < MAX_POINTS_DELTA_IN_METERS as f64
                })
            })
            .unwrap();

        let all_points = &all_points[0..=ty_idx];

        let mut common_points: Vec<&PointWithId> = Vec::new();

        for (idx, curr) in all_points.iter().enumerate() {
            let prev = if idx > 0 {
                all_points.get(idx - 1)
            } else {
                None
            };
            let next = all_points.get(idx + 1);

            if let Some(prev) = prev {
                if let Some(next) = next {
                    let point = Point::from(*curr);
                    let prev_point = Point::from(*prev);
                    let next_point = Point::from(*next);

                    let bearing_prev = point.haversine_bearing(prev_point);
                    let bearing_next = point.haversine_bearing(next_point);

                    // TODO: play with the MAX_BEARING value
                    if bearing_prev < MAX_BEARING && bearing_next < MAX_BEARING {
                        continue;
                    }
                }
            }
            common_points.push(curr)
        }

        let common_distance = LineString::from(common_points).haversine_length();

        CommonTrace {
            common_distance,
            common_start_point: PointOutput::from(all_points.first().unwrap().to_owned()),
            common_end_point: PointOutput::from(ty.to_owned()),
        }
    }
}

pub struct CommonTrace {
    pub common_distance: f64,
    pub common_start_point: PointOutput,
    pub common_end_point: PointOutput,
}

impl Trace<NotSimplified> {
    pub fn simplified(&self, epsilon: &f64) -> Trace<Simplified> {
        let linestring = LineString::from(self)
            .remove_repeated_points()
            .simplify(epsilon);

        self.from_linestring(linestring)
    }
}

impl From<Trace<Simplified>> for TraceOutput {
    fn from(value: Trace<Simplified>) -> Self {
        Self {
            id: value.id.to_owned(),
            points: value.points.iter().map(|p| p.id.to_owned()).collect(),
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
            id: value.id.to_string(),
            points,
            status: PhantomData,
        }
    }
}
