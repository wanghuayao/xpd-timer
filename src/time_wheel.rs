/// This module provides a time wheel implementation in Rust.
///
/// # Example
///
/// ```
/// use xpd_timer::time_wheel;
/// use std::time::Duration;
///
/// let (scheduler, receiver) = time_wheel(Duration::from_secs(1));
/// let scheduler = scheduler.arrange("task1");
/// scheduler.after(Duration::from_secs(5));
/// ```
///
/// In this example, a task named "task1" is scheduled to run 5 seconds later.
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

/// Scheduler struct, which schedules tasks to run at a specific time.
pub struct Scheduler<T> {
    handler: JoinHandle<()>,
    entities: Arc<Mutex<Vec<(T, SystemTime)>>>,
}

/// InnerScheduler struct, which is used to schedule tasks internally.
pub struct InnerScheduler<'a, T>(&'a Scheduler<T>, T);

impl<'a, T> InnerScheduler<'a, T> {
    /// Schedule a task to run at a specific time.
    pub fn at(self, when: SystemTime) {
        let InnerScheduler(scheduler, entity) = self;

        let mut entries = scheduler.entities.lock().unwrap();

        entries.push((entity, when));
        scheduler.handler.thread().unpark();
    }

    /// Schedule a task to run after a specific duration.
    pub fn after(self, after: Duration) {
        self.at(SystemTime::now() + after);
    }
}

impl<T> Scheduler<T> {
    /// Arrange a task to be scheduled.
    pub fn arrange(&self, entity: T) -> InnerScheduler<T> {
        return InnerScheduler(self, entity);
    }
}

/// TickReceiver struct, which receives ticks from the time wheel.
pub struct TickReceiver<T>(Receiver<T>);

impl<T> TickReceiver<T> {
    /// Receive a tick from the time wheel.
    pub fn recv(&self) -> TimerResult<T> {
        match self.0.recv() {
            Ok(result) => Ok(result),
            Err(err) => Err(TimerError::RecvError(err.to_string())),
        }
    }
}

/// Create a time wheel with a specific tick interval.
pub fn time_wheel<'a, T: Debug + Send + 'static>(
    interval: Duration,
) -> (Scheduler<T>, TickReceiver<T>) {
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

            let next_ticks = wheel.next_ticks();

            let next_tick_time = interval_in_nanos * next_ticks as u64;
            let process_time = one_loop_start.elapsed().as_nanos() as u64;

            if next_tick_time > process_time {
                thread::park_timeout(Duration::from_nanos(next_tick_time - process_time));
            }
        }
    });

    (Scheduler { handler, entities }, TickReceiver(receiver))
}
