use std::{
    fmt::Debug,
    mem,
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime},
};

use crate::core::Wheel;
use crate::{TimerError, TimerResult};

pub struct Scheduler<T> {
    handler: JoinHandle<()>,
    entities: Arc<Mutex<Vec<(T, SystemTime)>>>,
}

pub struct InnerScheduler<'a, T>(&'a Scheduler<T>, T);

impl<'a, T: Debug> InnerScheduler<'a, T> {
    pub fn at(self, when: SystemTime) {
        let InnerScheduler(scheduler, entity) = self;

        let mut entries = scheduler.entities.lock().unwrap();

        entries.push((entity, when));
        scheduler.handler.thread().unpark();
    }

    pub fn after(self, after: Duration) {
        self.at(SystemTime::now() + after);
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

    let entities = Arc::new(Mutex::new(Vec::<(T, SystemTime)>::new()));
    let std_tick_interval = interval.as_nanos() as u64;
    let entities_send = entities.clone();

    let handler = thread::spawn(move || {
        let mut wheel = Wheel::<T>::new();

        let start_at = Instant::now();
        thread::sleep(interval);

        loop {
            let real_ticks = wheel.ticks;
            let should_ticks = start_at.elapsed().as_nanos() / std_tick_interval as u128;

            if should_ticks > real_ticks as u128 {
                wheel.tick((should_ticks - real_ticks as u128) as u32, |entity| {
                    sender
                        .send(entity)
                        .expect("no reicver, stop running timer wheel");
                });
            }

            // resc
            let mut entities = entities_send.lock().unwrap();
            while let Some((entity, when)) = entities.pop() {
                let now = SystemTime::now();
                if when <= (now + interval) {
                    sender
                        .send(entity)
                        .expect("no reicver, stop running timer wheel");
                } else {
                    let offset = (when.duration_since(now).unwrap().as_nanos()
                        / std_tick_interval as u128) as u64;
                    wheel.schedule(entity, offset)
                }
            }
            mem::drop(entities);

            // park
            let next_tick = std_tick_interval * wheel.next_tick_times() as u64;
            thread::park_timeout(Duration::from_nanos(next_tick));
        }
    });

    (Scheduler { handler, entities }, TickReceiver(receiver))
}
