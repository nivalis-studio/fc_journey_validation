#[derive(thiserror::Error, Debug)]
pub enum JourneyValidationError {
    #[error("Missing startTime")]
    MissingStartTime,
    #[error("Missing endTime")]
    MissingEndTime,

    #[error("Missing driver")]
    MissingDriver,
    #[error("Missing passenger")]
    MissingPassenger,
    #[error("Driver is passenger")]
    InvalidPassenger,

    #[error("Too many traces")]
    TooManyTraces,

    #[error("No common points")]
    NoCommonPoints,

    #[error("Missing {0} trace")]
    MissingTrace(String),

    #[error("Empty {0} trace")]
    EmptyTrace(String),

    #[error("Start points timestamps are too far apart")]
    StartTimeDeltaTooBig,

    #[error("Not in France")]
    NotInFrance,

    #[error("Distance too {0}")]
    InvalidDistance(String),

    #[error("invalid json")]
    Serde(#[from] serde_json::Error),

    #[error("error while reading json file")]
    Io(#[from] std::io::Error),

    #[error("unexpected error")]
    Unexpected(#[from] anyhow::Error),
}
