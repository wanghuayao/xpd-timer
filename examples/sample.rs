use std::time::Duration;

use xpd_timer::{create_time_wheel, TimerResult};

#[allow(unused)]
fn main() -> TimerResult<()> {
    let (scheduler, receiver) = create_time_wheel::<String>(Duration::from_millis(1));

    let entity = "test".into();
    scheduler.arrange(entity).after(Duration::from_secs(5));

    let result = receiver.recv()?;

    println!("recived [{}]", result,);

    Ok(())
}
