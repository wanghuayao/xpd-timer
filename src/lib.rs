mod basic;
mod core;
mod time_wheel;

pub use crate::basic::*;
pub use time_wheel::{time_wheel, Scheduler, TickReceiver};

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        // let (sec, rec) = time_wheel(1000);
    }
}
