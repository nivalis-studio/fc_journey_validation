use crate::journeys::Trace;
use anyhow::Result;
use geo::FrechetDistance;

pub fn validate_traces(t1: Trace, t2: Trace) -> Result<f64> {
    let dist = t1.as_ref().frechet_distance(t2.as_ref());

    return Ok(dist);
}
