use crate::{error::JourneyValidationError, input::JourneyInput, trace::Trace};

pub struct Journey {
    pub driver_trace: Trace,
    pub passenger_trace: Trace,
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
