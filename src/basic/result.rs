use super::TimerError;

pub type TimerResult<R> = std::result::Result<R, TimerError>;
