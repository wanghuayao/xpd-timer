use std::fmt;

#[derive(Debug)]
pub enum TimerError {
    OutOfRangeError,
    RecvError(String),
    SendError(String),
    InternalError(String),
}

impl std::error::Error for TimerError {}

impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TimerError::OutOfRangeError => write!(f, "Over the size of timer"),
            TimerError::RecvError(msg) => write!(f, "Internal Error:{:?}", msg),
            TimerError::SendError(msg) => write!(f, "Internal Error:{:?}", msg),
            TimerError::InternalError(msg) => write!(f, "Internal Error:{}", msg),
        }
    }
}
