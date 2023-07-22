use std::time::{Duration, Instant};

use xpd_timer::{time_wheel, TimerResult};

#[allow(unused)]
fn main() -> TimerResult<()> {
    let (scheduler, receiver) = time_wheel::<String>(Duration::from_millis(1));

    let entity = "test".into();

    let start = Instant::now();
    scheduler.arrange(entity).after(Duration::from_secs(5));
    let result = receiver.recv()?;
    println!(
        "after {} millis recived [{}].",
        start.elapsed().as_millis(),
        result,
    );

    Ok(())
}
