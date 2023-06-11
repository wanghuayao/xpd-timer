mod basic;
mod core;
mod time_wheel;

pub use crate::basic::*;
pub use crate::core::SlotSize;
pub use time_wheel::{create_time_wheel, Scheduler, TickReceiver};

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        // let (sec, rec) = create_time_wheel(1000);
    }
}
