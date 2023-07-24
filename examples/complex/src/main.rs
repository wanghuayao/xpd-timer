use std::{
    thread,
    time::{Duration, SystemTime},
};

use rand::Rng;
use xpd_timer::{time_wheel, TimerResult};

#[derive(Debug)]
struct Entity {
    when_in_micros: u128,
    span: u64,
}

// #[allow(unused)]
fn main() -> TimerResult<()> {
    const UNIT: u64 = 1000;
    let (scheduler, receiver) = time_wheel::<Entity>(Duration::from_micros(UNIT));

    const TASK_CONT: u64 = 500;

    thread::spawn(move || {
        const MAX_DURATION_AS_MILLIN: u64 = 10000;

        let mut rng = rand::thread_rng();

        for i in 0..TASK_CONT {
            let millis: u64 = rng.gen_range(1..=MAX_DURATION_AS_MILLIN + (2 * i));

            let duration = Duration::from_millis(millis);

            let when = SystemTime::now() + duration;

            let when_in_micros = when
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros();

            let entity = Entity {
                when_in_micros,
                span: (millis * 1000) / UNIT,
            };

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
        let Entity {
            when_in_micros,
            span,
        } = receiver.recv()?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();

        if when_in_micros == now {
            // let dis = when_in_micros - now;
            // total_dis += dis;
            // println!("{}\tequal\t{} unit ({} micros)", i, dis / UNIT as u128, dis);
        } else if when_in_micros < now {
            let dis = now - when_in_micros;
            total_dis += dis;

            println!(
                "{}\tafter\t{} unit ({} micros), span:{}",
                i,
                dis / UNIT as u128,
                dis,
                span
            );
        } else {
            let dis = when_in_micros - now;
            total_dis += dis;
            println!(
                "{}\tbefore\t{} unit ({} micros), span:{}",
                i,
                dis / UNIT as u128,
                dis,
                span
            );
        }
    }

    println!("avg: {}", total_dis / TASK_CONT as u128);

    Ok(())
}
