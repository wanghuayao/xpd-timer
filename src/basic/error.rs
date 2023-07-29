use std::fmt;

#[derive(Debug)]
pub enum TimerError {
    RecvError(String),
    SendError(String),
}

impl std::error::Error for TimerError {}

impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TimerError::RecvError(msg) => write!(f, "Internal Error:{:?}", msg),
            TimerError::SendError(msg) => write!(f, "Internal Error:{:?}", msg),
        }
    }
}
