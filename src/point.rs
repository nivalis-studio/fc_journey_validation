use geo::{Coord, Point, Within};
use std::f64;

use crate::{france::FRANCE, input::PointInput};

#[derive(Debug)]
pub struct PointWithId {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

impl PointWithId {
    pub fn is_in_france(&self) -> bool {
        Point::from(self).is_within(&*FRANCE)
    }
}

impl From<&PointInput> for PointWithId {
    fn from(value: &PointInput) -> Self {
        Self {
            id: value.id.to_string(),
            x: value.longitude,
            y: value.latitude,
        }
    }
}

impl From<&PointWithId> for Point<f64> {
    fn from(value: &PointWithId) -> Self {
        Self::from(Coord::from(value))
    }
}

impl From<&PointWithId> for Coord<f64> {
    fn from(value: &PointWithId) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
