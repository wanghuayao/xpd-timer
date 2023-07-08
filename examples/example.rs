use std::time::{Duration, SystemTime};

use xpd_timer::{create_time_wheel, TimerResult};

#[derive(Debug)]
struct Item {
    content: String,
    when: SystemTime,
}

fn main() -> TimerResult<()> {
    let (scheduler, receiver) = create_time_wheel::<Item>(Duration::from_millis(1));

    println!("recv: {:?}", SystemTime::now());
    let five_senconds = Duration::from_secs(5);
    let now = SystemTime::now();
    let when = now + five_senconds;

    let item = Item {
        content: "test".to_string(),
        when,
    };
    scheduler.schedule(item, five_senconds);

    let item = receiver.recv()?;

    println!(
        "recv: {:?}",
        SystemTime::now().duration_since(now).unwrap().as_millis()
    );

    Ok(())
}
