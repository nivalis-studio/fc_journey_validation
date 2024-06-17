use std::{collections::HashMap, f64, isize, marker::PhantomData, num::Wrapping};

use chrono::{DateTime, Utc};
use geo::{
    Coord, FrechetDistance, HaversineDistance, HaversineLength, LineString, Point,
    RemoveRepeatedPoints, Simplify,
};

use crate::{
    input::TraceInput,
    output::{PointOutput, TraceOutput},
    point::PointWithId,
};

const MAX_POINTS_DELTA_IN_METERS: u16 = 1000;

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

        let mut homogenous_segments: Vec<Vec<&PointWithId>> = Vec::new();
        let mut mixed_segments: Vec<Vec<&PointWithId>> = Vec::new();

        for (idx, curr) in all_points.iter().enumerate() {
            let prev = if idx > 0 {
                all_points.get(idx - 1)
            } else {
                None
            };
            let prev_prev = if idx > 1 {
                all_points.get(idx - 2)
            } else {
                None
            };
            let next = all_points.get(idx + 1);

            if let Some(prev) = prev {
                if curr.trace_id == prev.trace_id {
                    homogenous_segments.last_mut().unwrap().push(curr);
                    continue;
                }
            }

            if let Some(next) = next {
                if curr.trace_id == next.trace_id {
                    homogenous_segments.push(vec![curr]);
                    continue;
                }
            }

            if let Some(prev) = prev {
                if let Some(prev_prev) = prev_prev {
                    if prev_prev.trace_id == prev.trace_id {
                        mixed_segments.push(vec![curr]);
                        continue;
                    }
                }
            }

            if let Some(segment) = mixed_segments.last_mut() {
                segment.push(curr);
                continue;
            }

            mixed_segments.push(vec![curr]);
        }

        let homogenous_distance = get_segments_length(homogenous_segments);
        // FIXME: calculate mixed distance more precisely
        let mixed_distance = get_segments_length(mixed_segments);
        let common_distance = homogenous_distance + mixed_distance;

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

fn get_segments_length(segments: Vec<Vec<&PointWithId>>) -> f64 {
    segments
        .iter()
        .map(|v| {
            LineString::from(v.iter().map(|p| Coord::from(*p)).collect::<Vec<Coord>>())
                .haversine_length()
        })
        .sum()
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

impl From<&Trace> for LineString<f64> {
    fn from(value: &Trace) -> Self {
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
