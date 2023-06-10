use std::{
    cmp::Reverse,
    time::{Duration, SystemTime},
};

use xpd_timer::{create_time_wheel, TimerResult};

#[derive(Debug)]
struct Item {
    content: String,
    when: SystemTime,
}

fn main() -> TimerResult<()> {
    let (scheduler, receiver) = create_time_wheel::<Item>(Duration::from_millis(256));

    let five_senconds = Duration::from_secs(5);
    let when = SystemTime::now() + five_senconds;

    let item = Item {
        content: "test".to_string(),
        when,
    };
    scheduler.schedule(item, five_senconds)?;

    let a = receiver.recv();
    println!("recv: {:?}", a);

    Ok(())
}
