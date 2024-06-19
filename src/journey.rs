use crate::{
    error::JourneyValidationError,
    input::JourneyInput,
    output::{Output, TracesOutput},
    trace::{CommonTrace, Trace},
    Result,
};

const MAX_DELTA_IN_MILLISECONDS: i64 = 90_000;
const SIMPLIFY_EPSILON: f64 = 0.00001;
const MAX_DISTANCE: f64 = 80_000.0;
const MIN_DISTANCE: f64 = 1_000.0;

pub struct Journey {
    pub driver_trace: Trace,
    pub passenger_trace: Trace,
}

impl Journey {
    pub fn validate(&self) -> Output {
        if let Err(err) = self.validate_edges() {
            return Output::from(err);
        }

        let CommonTrace {
            common_distance,
            common_start_point,
            common_end_point,
        } = match self.driver_trace.common_trace_with(&self.passenger_trace) {
            Ok(common_trace) => common_trace,
            Err(err) => return Output::from(err),
        };

        if common_distance < MIN_DISTANCE {
            return Output::from(JourneyValidationError::InvalidDistance("short".into()));
        }

        if common_distance > MAX_DISTANCE {
            return Output::from(JourneyValidationError::InvalidDistance("long".into()));
        }

        let driver_trace = self.driver_trace.simplified(SIMPLIFY_EPSILON);
        let passenger_trace = self.driver_trace.simplified(SIMPLIFY_EPSILON);
        let average_confidence = driver_trace.confidence_with(&passenger_trace);

        Output::Success(crate::output::OutputSuccess {
            average_confidence,
            traces: TracesOutput {
                passenger_trace: passenger_trace.into(),
                driver_trace: driver_trace.into(),
            },
            common_distance,
            common_start_point,
            common_end_point,
        })
    }

    pub fn validate_edges(&self) -> Result<()> {
        let (driver_start, driver_end) = self.driver_trace.get_edges();
        let (passenger_start, passenger_end) = self.passenger_trace.get_edges();

        if driver_start.get_ms_delta_with(passenger_start) > MAX_DELTA_IN_MILLISECONDS {
            return Err(JourneyValidationError::StartTimeDeltaTooBig);
        }

        if ![driver_start, driver_end, passenger_start, passenger_end]
            .iter()
            .any(|p| p.is_in_france())
        {
            return Err(JourneyValidationError::NotInFrance);
        }

        Ok(())
    }
}

impl TryFrom<JourneyInput> for Journey {
    type Error = JourneyValidationError;

    fn try_from(journey: JourneyInput) -> Result<Self, Self::Error> {
        journey
            .start_time
            .ok_or(JourneyValidationError::MissingStartTime)?;

        journey
            .end_time
            .ok_or(JourneyValidationError::MissingEndTime)?;

        let driver_id = journey
            .driver_id
            .as_ref()
            .ok_or(JourneyValidationError::MissingDriver)?;

        let passenger_id = journey
            .passenger_id
            .as_ref()
            .ok_or(JourneyValidationError::MissingPassenger)?;

        if passenger_id == driver_id {
            return Err(JourneyValidationError::InvalidPassenger);
        }

        let driver_trace = journey
            .gps_trace
            .iter()
            .find(|t| t.user_id.as_ref() == Some(driver_id))
            .ok_or(JourneyValidationError::MissingTrace("driver".into()))?;

        let passenger_trace = journey
            .gps_trace
            .iter()
            .find(|t| t.user_id.as_ref() == Some(passenger_id))
            .ok_or(JourneyValidationError::MissingTrace("passenger".into()))?;

        if driver_trace.points.len() < 2 {
            return Err(JourneyValidationError::EmptyTrace("driver".into()));
        }

        if passenger_trace.points.len() < 2 {
            return Err(JourneyValidationError::EmptyTrace("passenger".into()));
        }

        Ok(Self {
            driver_trace: driver_trace.into(),
            passenger_trace: passenger_trace.into(),
        })
    }
}
