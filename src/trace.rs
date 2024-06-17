use std::{collections::HashMap, f64, marker::PhantomData};

use geo::{
    Closest, HaversineClosestPoint, HaversineDistance, HaversineLength, LineString, Point,
    RemoveRepeatedPoints, Simplify,
};

use crate::{input::TraceInput, point::PointWithId};

const MAX_POINTS_DELTA_IN_METERS: u16 = 1000;

pub struct Simplified;
pub struct NotSimplified;

#[derive(Debug)]
pub struct Trace<T = NotSimplified> {
    pub points: Vec<PointWithId>,
    status: PhantomData<T>,
}

impl Trace {
    pub fn from_linestring<T>(&self, linestring: LineString<f64>) -> Trace<T> {
        let coords: HashMap<(u64, u64), String> = self
            .points
            .iter()
            .map(|p| ((p.x.to_bits(), p.y.to_bits()), p.id.to_string()))
            .collect();

        let points = linestring
            .into_iter()
            .filter_map(|coord| {
                coords
                    .get(&(coord.x.to_bits(), coord.y.to_bits()))
                    .map(|id| PointWithId {
                        id: id.to_string(),
                        x: coord.x,
                        y: coord.y,
                    })
            })
            .collect();

        Trace {
            points,
            status: PhantomData,
        }
    }

    pub fn haversine_length(&self) -> f64 {
        let linestring = LineString::from(self);
        linestring.haversine_length()
    }

    pub fn get_common_linestring_with(&self, other: &Trace) -> LineString {
        let (shortest, longest) = if self.points.len() < other.points.len() {
            (self, other)
        } else {
            (other, self)
        };

        let longest_linestring = LineString::from(longest);
        let shortest_linestring = LineString::from(shortest);

        let common_linestring: LineString = shortest_linestring
            .into_iter()
            .filter(|coord| {
                let point: Point<f64> = Point::new(coord.x, coord.y);

                let closest_point: Closest<f64> =
                    longest_linestring.haversine_closest_point(&point);

                let other_point: Point<f64> = match closest_point {
                    Closest::SinglePoint(point) => point,
                    Closest::Intersection(intersection) => intersection,
                    Closest::Indeterminate => return false,
                };

                let dist = point.haversine_distance(&other_point);

                dist < MAX_POINTS_DELTA_IN_METERS as f64
            })
            .collect();

        common_linestring
    }
}

impl Trace<NotSimplified> {
    pub fn simplified(&self, epsilon: &f64) -> Trace<Simplified> {
        let linestring = LineString::from(self)
            .remove_repeated_points()
            .simplify(epsilon);

        self.from_linestring(linestring)
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

impl From<TraceInput> for Trace {
    fn from(value: TraceInput) -> Self {
        let points = value.points.iter().map(PointWithId::from).collect();

        Self {
            points,
            status: PhantomData,
        }
    }
}
