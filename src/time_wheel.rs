use std::{
    fmt::Debug,
    mem,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime},
};

use crossbeam_channel::Receiver;

use crate::core::Wheel;
use crate::{TimerError, TimerResult};

pub struct Scheduler<T> {
    handler: JoinHandle<()>,
    entities: Arc<Mutex<Vec<(T, SystemTime)>>>,
}

pub struct InnerScheduler<'a, T>(&'a Scheduler<T>, T);

impl<'a, T> InnerScheduler<'a, T> {
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

impl<T> Scheduler<T> {
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

pub fn time_wheel<'a, T: Debug + Send + 'static>(
    interval: Duration,
) -> (Scheduler<T>, TickReceiver<T>) {
    // let (sender, receiver) = channel::<T>();

    let (sender, receiver) = crossbeam_channel::unbounded();

    let entities = Arc::new(Mutex::new(Vec::<(T, SystemTime)>::new()));
    let interval_in_nanos = interval.as_nanos() as u64;
    let entities_send = entities.clone();

    let handler = thread::spawn(move || {
        let notice = move |entity| {
            sender
                .send(entity)
                .expect("no receiver, stop running timer wheel");
        };

        let notice_copy = notice.clone();

        let mut wheel = Wheel::<T>::new(notice);

        let start = Instant::now();
        let start_at = SystemTime::now();

        thread::sleep(interval);
        loop {
            let real_ticks = wheel.ticks as u128;
            let should_ticks = start.elapsed().as_nanos() / interval_in_nanos as u128;

            let one_loop_start = Instant::now();
            if should_ticks > real_ticks {
                wheel.tick_to(should_ticks as u64);
            }

            // resc
            let mut entities = entities_send.lock().unwrap();
            while let Some((entity, when)) = entities.pop() {
                let pure_time_offset = when
                    .duration_since(start_at)
                    .unwrap_or_default()
                    .checked_sub(start.elapsed())
                    .unwrap_or_default()
                    .as_nanos();

                if pure_time_offset > interval_in_nanos as u128 {
                    let offset = pure_time_offset / interval_in_nanos as u128;
                    wheel.schedule(entity, offset as u64, when);
                } else {
                    notice_copy(entity);
                }
            }
            mem::drop(entities);

            // park
            let next_ticks = wheel.next_ticks();

            let next_tick_time = interval_in_nanos * next_ticks as u64;
            let process_time = one_loop_start.elapsed().as_nanos() as u64;

            println!(
                "next_ticks:{}, next_tick_time,{}, process_time:{}",
                next_ticks, next_tick_time, process_time
            );

            if next_tick_time > process_time {
                thread::park_timeout(Duration::from_nanos(next_tick_time - process_time));
            }
        }
    });

    (Scheduler { handler, entities }, TickReceiver(receiver))
}
