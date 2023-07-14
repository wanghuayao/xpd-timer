use std::time::{Duration, Instant};

use xpd_timer::{create_time_wheel, TimerResult};

#[allow(unused)]
fn main() -> TimerResult<()> {
    let (scheduler, receiver) = create_time_wheel::<String>(Duration::from_micros(512));

    let start = Instant::now();

    let entity = "test".into();
    scheduler.arrange(entity).after(Duration::from_secs(5));

    let result = receiver.recv()?;

    println!(
        "recived [{}] after {} millis",
        result,
        start.elapsed().as_millis()
    );

    Ok(())
}
