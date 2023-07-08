use std::{
    borrow::BorrowMut,
    fmt::Debug,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime},
};

use crate::core::Wheel;
use crate::{TimerError, TimerResult};

pub struct Scheduler<T> {
    // duration between tow tick(in milliseconds)
    milli_interval: u128,
    sender: Sender<T>,
    wheel: Arc<Mutex<Wheel<T>>>,
}

impl<T: Debug> Scheduler<T> {
    pub fn schedule_at(&self, content: T, when: SystemTime) {
        let now = SystemTime::now();
        let after = if when > now {
            when.duration_since(now).unwrap()
        } else {
            // 'when' is the past time
            Duration::from_millis(0)
        };

        self.schedule(content, after)
    }
    pub fn schedule(&self, content: T, after: Duration) {
        if after.is_zero() {
            self.sender
                .send(content)
                .or_else(|err| Err(TimerError::SendError(err.to_string())))
                .unwrap();

            return;
        }

        let tick_times = after.as_millis() / self.milli_interval;
        let mut wheel = self.wheel.lock().unwrap();
        wheel.borrow_mut().schedule(content, tick_times);
    }
}

pub struct TickReceiver<T>(Receiver<T>);

impl<T> TickReceiver<T> {
    pub fn recv(&self) -> TimerResult<T> {
        match self.0.recv() {
            Ok(result) => Ok(result),
            Err(err) => Err(TimerError::SendError(err.to_string())),
        }
    }
}

pub fn create_time_wheel<T: Debug + Send + 'static>(
    interval: Duration,
) -> (Scheduler<T>, TickReceiver<T>) {
    let (sender, receiver) = channel::<T>();

    let wheel = Arc::new(Mutex::new(Wheel::new()));

    let start_at = Instant::now();
    let milli_interval = interval.as_millis();

    let wheel_send = Arc::clone(&wheel);
    let sender_send = Sender::clone(&sender); //同一通道，增加一个发送者

    let _handler = thread::spawn(move || {
        thread::sleep(interval);

        loop {
            let now = Instant::now();
            let mut wheel = wheel_send.lock().unwrap();

            let need_tick_time = start_at.elapsed().as_millis() / milli_interval;

            while need_tick_time > wheel.tick_times as u128 {
                let times = (need_tick_time - wheel.tick_times as u128) as u32;
                let _ = wheel.tick(times, |item| {
                    if let Err(err) = sender_send.send(item) {
                        println!("Warning: {:?}", err);
                    }
                });
            }

            let process_time = now.elapsed().as_millis();
            if milli_interval > process_time {
                thread::sleep(Duration::from_millis(
                    (milli_interval - process_time) as u64,
                ));
            }
        }
    });

    (
        Scheduler {
            sender,
            wheel,
            milli_interval,
        },
        TickReceiver(receiver),
    )
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rand::Rng;

    #[test]
    fn it_works() {
        use super::create_time_wheel;
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        const INTERVAL: u64 = 60;

        let (scheduler, receiver) = create_time_wheel::<String>(Duration::from_millis(INTERVAL));

        const CNT: usize = 1000;

        const MAX_DURATION: u64 = 50000;

        let mut rng = rand::thread_rng();
        let now_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        for _i in 0..CNT {
            let millis: u64 = rng.gen_range(1..=MAX_DURATION);
            let _ = scheduler.schedule(
                (now_millis + millis as u128).to_string(),
                Duration::from_millis(millis),
            );
        }

        let start = Instant::now();
        let mut count = 0;
        loop {
            let content = receiver.recv().unwrap();

            println!(
                "expend:{}",
                Instant::now().duration_since(start).as_millis()
            );

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let expect_time: u128 = content.parse().unwrap();

            let distance = if expect_time > now {
                // expect_time.wrapping_sub(now)
                expect_time - now
            } else {
                // now.wrapping_sub(expect_time)
                now - expect_time
            } as u64;

            if distance > 0 {
                println!(
                    "{}, exp : {}, now:{},distance:{}",
                    count, expect_time, now, distance
                );
            }

            assert!(distance <= INTERVAL);

            count += 1;
            if count >= CNT {
                break;
            }
        }
    }

    #[test]
    fn past_time() {
        use super::create_time_wheel;
        use std::time::{Duration, SystemTime};

        const INTERVAL: u64 = 16;

        let (scheduler, receiver) = create_time_wheel::<String>(Duration::from_millis(INTERVAL));

        let when = SystemTime::now() - Duration::from_secs(100);
        scheduler.schedule_at("test".to_string(), when);

        let content = receiver.recv().unwrap();
        assert_eq!(content, "test");
    }
}
