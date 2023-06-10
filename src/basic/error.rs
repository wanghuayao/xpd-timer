use std::{fmt, sync::mpsc::RecvError};

#[derive(Debug)]
pub enum TimerError {
    OutOfRangeError,
    RecvieError(RecvError),
    InternalError(String),
}

pub type TimerResult<R> = std::result::Result<R, TimerError>;

impl std::error::Error for TimerError {}

impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TimerError::OutOfRangeError => write!(f, "Over the size of timer"),
            TimerError::RecvieError(err) => write!(f, "Internal Error:{:?}", err),
            TimerError::InternalError(message) => write!(f, "Internal Error:{}", message),
        }
    }
}
