use std::{
    thread,
    time::{Duration, SystemTime},
};

use rand::Rng;
use xpd_timer::{create_time_wheel, TimerResult};

// #[allow(unused)]
fn main() -> TimerResult<()> {
    let (scheduler, receiver) = create_time_wheel::<u128>(Duration::from_micros(1));

    const TASK_CONT: u64 = 500;

    thread::spawn(move || {
        const MAX_DURATION_AS_MILLIN: u64 = 10000;

        let mut rng = rand::thread_rng();

        for i in 0..TASK_CONT {
            let millis: u64 = rng.gen_range(1..=MAX_DURATION_AS_MILLIN + (2 * i));

            let duration = Duration::from_millis(millis);

            let when = SystemTime::now() + duration;

            let entity = when
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();

            if rng.gen_range(100..=2000) % 2 == 0 {
                scheduler.arrange(entity).at(when);
            } else {
                scheduler.arrange(entity).after(duration);
            }

            thread::sleep(Duration::from_millis(rng.gen_range(100..=2000)))
        }
    });

    for i in 0..TASK_CONT {
        let result = receiver.recv()?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();

        if result == now {
            println!("{}\tequal\t{}\tmillis", i, (result - now) / 1000);
        } else if result < now {
            println!("{}\tafter\t{}\tmillis", i, (now - result) / 1000);
        } else {
            println!("{}\tbefor\t{}\tmillis", i, (result - now) / 1000);
        }
    }
    Ok(())
}
