use std::{
    borrow::BorrowMut,
    fmt::Debug,
    mem,
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

pub struct InnerScheduler<'a, T>(&'a Scheduler<T>, T);

impl<'a, T: Debug> InnerScheduler<'a, T> {
    pub fn at(self, when: SystemTime) {
        let now = SystemTime::now();
        let after = if when > now {
            when.duration_since(now).unwrap()
        } else {
            // 'when' is the past time
            Duration::from_nanos(0)
        };

        self.after(after);
    }
    pub fn after(self, after: Duration) {
        let InnerScheduler(scheduler, entity) = self;

        let after_in_nanos = after.as_nanos();
        if after_in_nanos < scheduler.std_tick_interval {
            scheduler
                .sender
                .send(entity)
                .or_else(|err| Err(TimerError::SendError(err.to_string())))
                .unwrap();

            return;
        }

        // TODO: u64 is ength?
        let offset = (after.as_nanos() / scheduler.std_tick_interval) as u64;

        let mut wheel = scheduler.wheel.lock().unwrap();
        wheel.borrow_mut().schedule(entity, offset);
        scheduler.handler.thread().unpark();
    }
}

impl<T: Debug> Scheduler<T> {
    pub fn arrange(&self, entity: T) -> InnerScheduler<T> {
        return InnerScheduler(self, entity);
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

    let std_tick_interval = interval.as_nanos();

    let wheel_send = Arc::clone(&wheel);
    let sender_send = sender.clone(); // Clone the sender for the same channel

    let handler = thread::spawn(move || {
        let start_at = Instant::now();
        thread::sleep(interval);

        loop {
            let mut wheel = wheel_send.lock().unwrap();

            let mut need_tick_times = start_at.elapsed().as_nanos() / std_tick_interval;
            while need_tick_times > wheel.tick_times as u128 {
                let mut times = (need_tick_times - wheel.tick_times as u128) as u32;

                while times > 0 {
                    let real_times = wheel.tick(times, |item| {
                        if let Err(err) = sender_send.send(item) {
                            println!("Warning: {:?}", err);
                        }
                    });

                    times -= real_times;
                }

                need_tick_times = start_at.elapsed().as_nanos() / std_tick_interval;
            }

            let next_tick_times = wheel.next_tick_times();
            let next_tick = std_tick_interval * next_tick_times as u128;
            // let process_time = now.elapsed().as_nanos();
            if next_tick > 0 {
                let park_duration = Duration::from_nanos((next_tick) as u64);
                mem::drop(wheel);
                thread::park_timeout(park_duration);
            }
        }
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
