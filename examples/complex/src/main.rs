use std::{
    thread,
    time::{Duration, SystemTime},
};

use rand::Rng;
use xpd_timer::{time_wheel, TimerResult};

// #[allow(unused)]
fn main() -> TimerResult<()> {
    let (scheduler, receiver) = time_wheel::<u128>(Duration::from_micros(512));

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

    let mut total_dis = 0u128;

    for i in 0..TASK_CONT {
        let result = receiver.recv()?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();

        if result == now {
            let dis = result - now;
            total_dis += dis;
            println!("{}\tequal\t{} unit ({} micros)", i, dis / 512, dis);
        } else if result < now {
            let dis = now - result;
            total_dis += dis;

            println!("{}\tafter\t{} unit ({} micros)", i, dis / 512, dis);
        } else {
            let dis = result - now;
            total_dis += dis;
            println!("{}\tbefore\t{} unit ({} micros)", i, dis / 512, dis);
        }
    }

    println!("avg: {}", total_dis / TASK_CONT as u128);

    Ok(())
}
