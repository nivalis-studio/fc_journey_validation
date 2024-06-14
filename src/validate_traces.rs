use anyhow::Result;

use crate::traces::Trace;

pub fn validate_traces(t1: &Trace, t2: &Trace) -> Result<f64> {
    let dist = t1.compare_distance(t2);

    Ok(dist)
}
