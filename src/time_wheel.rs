use std::{
    borrow::BorrowMut,
    fmt::Debug,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime},
};

use crate::core::Wheel;
use crate::{TimerError, TimerResult};

pub struct Scheduler<T> {
    // duration between two ticks(in nanos)
    std_tick_interval: u128,
    sender: Sender<T>,
    wheel: Arc<Mutex<Wheel<T>>>,
    handler: JoinHandle<()>,
}

impl<'a, T: Debug> Scheduler<T> {
    pub fn schedule_at(&self, content: T, when: SystemTime) {
        let now = SystemTime::now();
        let after = if when > now {
            when.duration_since(now).unwrap()
        } else {
            // 'when' is the past time
            Duration::from_nanos(0)
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

        let tick_times = after.as_nanos() / self.std_tick_interval;
        let mut wheel = self.wheel.lock().unwrap();

        wheel.borrow_mut().schedule(content, tick_times);

        self.handler.thread().unpark();
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

pub fn create_time_wheel<'a, T: Debug + Send + 'static>(
    interval: Duration,
) -> (Scheduler<T>, TickReceiver<T>) {
    let (sender, receiver) = channel::<T>();

    let wheel = Arc::new(Mutex::new(Wheel::new()));

    let start_at = Instant::now();
    let std_tick_interval = interval.as_nanos();

    let wheel_send = Arc::clone(&wheel);
    let sender_send = sender.clone(); // Clone the sender for the same channel

    let handler = thread::spawn(move || {
        thread::sleep(interval);

        loop {
            let now = Instant::now();
            let mut wheel = wheel_send.lock().unwrap();

            let need_tick_times = start_at.elapsed().as_nanos() / std_tick_interval;

            while need_tick_times > wheel.tick_times as u128 {
                let times = (need_tick_times - wheel.tick_times as u128) as u32;

                let _ = wheel.tick(times, |item| {
                    if let Err(err) = sender_send.send(item) {
                        println!("Warning: {:?}", err);
                    }
                });
            }

            let next_tick = std_tick_interval * wheel.next_tick_times() as u128;
            let process_time = now.elapsed().as_nanos();
            if next_tick > process_time {
                thread::park_timeout(Duration::from_nanos((next_tick - process_time) as u64));
            }
        }
        #[allow(unreachable_code)]
        ()
    });

    (
        Scheduler {
            sender,
            wheel,
            std_tick_interval,
            handler,
        },
        TickReceiver(receiver),
    )
}
