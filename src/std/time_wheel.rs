use std::thread;
use std::{
    borrow::BorrowMut,
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
};

use std::time::{Duration, Instant, SystemTime};

use crate::core::{SlotSize, Wheel};

pub struct Scheduler<T> {
    // duration between tow tick(in milliseconds)
    milli_interval: u128,
    wheel: Arc<Mutex<Wheel<T>>>,
}

impl<T> Scheduler<T> {
    pub fn schedule_at(&self, content: T, when: SystemTime) -> Result<(), String> {
        let after = when.duration_since(SystemTime::now());
        if after.is_err() {
            return Err("can't asign a past time".to_string());
        }

        self.schedule(content, after.unwrap())
    }
    pub fn schedule(&self, content: T, after: Duration) -> Result<(), String> {
        let tick_times = after.as_millis() / self.milli_interval;
        let mut wheel = self.wheel.lock().unwrap();
        wheel.borrow_mut().schedule(content, tick_times)
    }
}

pub struct TickReceiver<T>(Receiver<T>);

impl<T> TickReceiver<T> {
    pub fn recv(&self) -> Result<T, String> {
        match self.0.recv() {
            Ok(result) => Ok(result),
            Err(err) => Err(err.to_string()),
        }
    }

    // TODO
    pub fn stop(&self) -> Result<T, String> {
        match self.0.recv() {
            Ok(result) => Ok(result),
            Err(err) => Err(err.to_string()),
        }
    }
}

pub fn create_time_wheel<T: Send + 'static>(interval: Duration) -> (Scheduler<T>, TickReceiver<T>) {
    let (sender, receiver) = channel::<T>();

    let wheel = Arc::new(Mutex::new(Wheel::new(SlotSize::Normal)));

    let start_at = Instant::now();
    let milli_interval = interval.as_millis();

    let wheel_send = Arc::clone(&wheel);

    let _handler = thread::spawn(move || {
        thread::sleep(interval);

        loop {
            let now = Instant::now();
            let mut wheel = wheel_send.lock().unwrap();

            let need_tick_time = start_at.elapsed().as_millis() / milli_interval;

            while need_tick_time - wheel.tick_times as u128 > 0 {
                if let Some(items) = wheel.tick() {
                    for item in items {
                        sender.send(item.data).unwrap();
                    }
                }
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
            wheel,
            milli_interval,
        },
        TickReceiver(receiver),
    )
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    #[test]
    fn it_works() {
        use crate::std::create_time_wheel;
        use std::time::{Duration, SystemTime, UNIX_EPOCH};

        const INTERVAL: u64 = 16;

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

        let mut count = 0;
        loop {
            let content = receiver.recv().unwrap();
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
}
