use std::{
    thread,
    time::{Duration, Instant, SystemTime},
};

use rand::Rng;
use xpd_timer::{create_time_wheel, TimerResult};

// #[allow(unused)]
fn main() -> TimerResult<()> {
    let (scheduler, receiver) = create_time_wheel::<u128>(Duration::from_millis(1));

    const TASK_CONT: usize = 2;

    thread::spawn(move || {
        const MAX_DURATION_AS_MILLIN: u64 = 5000;

        let mut rng = rand::thread_rng();

        for _i in 0..TASK_CONT {
            let millis: u64 = rng.gen_range(1..=MAX_DURATION_AS_MILLIN);

            let when = SystemTime::now() + Duration::from_millis(millis);

            let entity = when
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();

            println!("millis:{} when:{}", millis, entity);

            scheduler.arrange(entity).at(when);

            thread::sleep(Duration::from_millis(rng.gen_range(100..=2000)))
        }
    });

    for _ in 0..TASK_CONT {
        let result = receiver.recv()?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        if result == now {
            println!("equal\t{}\tmillis", result - now);
        } else if result < now {
            println!("after\t{}\tmillis", now - result);
        } else {
            println!("befor\t{}\tmillis", result - now);
        }
    }
    Ok(())
}
