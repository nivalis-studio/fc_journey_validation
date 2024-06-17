use crate::{error::JourneyValidationError, input::JourneyInput, trace::Trace, Result};

const MAX_DELTA_IN_MILLISECONDS: i64 = 90_000;

pub struct Journey {
    pub driver_trace: Trace,
    pub passenger_trace: Trace,
}

impl Journey {
    pub fn validate(&self) -> Result<()> {
        self.validate_edges()?;

        Ok(())
    }

    pub fn validate_edges(&self) -> Result<()> {
        let Self {
            driver_trace,
            passenger_trace,
        } = self;

        let (driver_start, driver_end) = driver_trace.get_edges();
        let (passenger_start, passenger_end) = passenger_trace.get_edges();

        for ((first, second), name) in [
            ((driver_start, passenger_start), "start"),
            ((driver_end, passenger_end), "end"),
        ]
        .iter()
        {
            if first.get_ms_delta_with(second) > MAX_DELTA_IN_MILLISECONDS {
                return Err(JourneyValidationError::TimestampsDeltaTooBig(
                    name.to_string(),
                ));
            }
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
        if journey.start_time.is_none() {
            return Err(JourneyValidationError::MissingStartTime);
        }
        if journey.end_time.is_none() {
            return Err(JourneyValidationError::MissingEndTime);
        }

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
            .find(|t| t.user_id.as_str() == driver_id)
            .ok_or(JourneyValidationError::MissingTrace("driver".into()))?;

        let passenger_trace = journey
            .gps_trace
            .iter()
            .find(|t| t.user_id.as_str() == passenger_id)
            .ok_or(JourneyValidationError::MissingTrace("passenger_id".into()))?;

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
