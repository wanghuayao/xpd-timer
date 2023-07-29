use timed::timed;

use super::{bucket::Bucket, slot::Entity};
use std::{
    convert::TryInto,
    fmt::Debug,
    time::{SystemTime, UNIX_EPOCH},
};

const LEVEL_COUNT: usize = 6;

pub struct Wheel<T> {
    buckets: [Bucket<T>; LEVEL_COUNT],
    pub(crate) ticks: u64,
    homeless: Option<Vec<Entity<T>>>,
    _notice: Box<dyn Fn(T)>,
}

impl<T: Debug> Wheel<T> {
    pub(crate) fn new(notice: impl Fn(T) + 'static) -> Self {
        let buckets = (0..LEVEL_COUNT)
            .map(|level| Bucket::new(level as u32))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Wheel {
            buckets,
            ticks: 0,
            homeless: None,
            _notice: Box::new(notice),
        }
    }

    #[timed]
    pub(crate) fn schedule(&mut self, entity: T, offset: u64, when: SystemTime) {
        let entity = Entity {
            data: entity,
            tick_times: offset + self.ticks,
            when,
            offset: offset,
            ticks: self.ticks,
        };

        match to_level(offset) {
            Some(level) => self.buckets[level].add(entity, offset as u64),
            _ => self.homeless.get_or_insert_with(Vec::new).push(entity),
        }
    }

    #[timed]
    pub(crate) fn tick_to(&mut self, ticks: u64) {
        if ticks <= self.ticks {
            return;
        }
        let mut times = (ticks - self.ticks) as u32;

        self.ticks = ticks;

        const MAX_LEVEL_INDEX: usize = LEVEL_COUNT - 1;
        let mut tick_to_max_level = false;

        for level in 0..LEVEL_COUNT {
            let (result, next_level_tick_times) = self.buckets[level].tick(times);
            result.map(|entities| self.dispose_of(entities));
            tick_to_max_level = level == MAX_LEVEL_INDEX;
            if next_level_tick_times == 0 {
                break;
            }
            // prepare next loop
            times = next_level_tick_times;
        }

        if tick_to_max_level {
            self.homeless
                .take()
                .map(|entities| self.dispose_of(entities));
        }
    }

    pub(crate) fn next_ticks(&self) -> u32 {
        let mut next_ticks = 1;

        for level in 0..4 {
            let (non_stop_ticks, is_need_check_next) = self.buckets[level].non_stop_ticks();
            next_ticks = next_ticks.max(non_stop_ticks);

            if !is_need_check_next {
                break;
            }
        }
        next_ticks
    }

    fn dispose_of(&mut self, entities: Vec<Entity<T>>) {
        let ticks = self.ticks;
        for entity in entities {
            if entity.tick_times <= ticks as u64 {
                self.notice(entity);
            } else {
                // add to wheel again
                let offset = entity.tick_times - ticks;
                let level = to_level(offset);
                self.buckets[level.unwrap()].add(entity, offset);
            }
        }
    }

    fn notice(&self, entity: Entity<T>) {
        let now = SystemTime::now();
        let when = entity.when;

        let mut dis = 0u128;
        if when > now {
            dis = when.duration_since(now).unwrap().as_micros();
        } else {
            dis = now.duration_since(when).unwrap().as_micros();
        }

        println!(
            "notice entity ticks: {}, system ticks:{}, time diff: {}, add offset:{}, ticks:{}",
            entity.tick_times, self.ticks, dis, entity.offset, entity.ticks
        );

        assert!(self.ticks >= entity.tick_times);

        (self._notice)(entity.data);
    }
}

fn to_level(offset: u64) -> Option<usize> {
    const SIZE_OF_LEVEL_0: u64 = 1 << (6 * 1);
    const SIZE_OF_LEVEL_1: u64 = 1 << (6 * 2);
    const SIZE_OF_LEVEL_2: u64 = 1 << (6 * 3);
    const SIZE_OF_LEVEL_3: u64 = 1 << (6 * 4);
    const SIZE_OF_LEVEL_4: u64 = 1 << (6 * 5);
    const SIZE_OF_LEVEL_5: u64 = 1 << (6 * 6);
    match offset {
        t if t < SIZE_OF_LEVEL_0 => Some(0),
        t if t < SIZE_OF_LEVEL_1 => Some(1),
        t if t < SIZE_OF_LEVEL_2 => Some(2),
        t if t < SIZE_OF_LEVEL_3 => Some(3),
        t if t < SIZE_OF_LEVEL_4 => Some(4),
        t if t < SIZE_OF_LEVEL_5 => Some(5),
        _ => None,
    }

    // const SIZE_OF_LEVEL: [u64; 6] = [
    //     1 << (6 * 1),
    //     1 << (6 * 2),
    //     1 << (6 * 3),
    //     1 << (6 * 4),
    //     1 << (6 * 5),
    //     1 << (6 * 6),
    // ];
    // SIZE_OF_LEVEL.iter().position(|&x| offset < x)
}

fn _current_millis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_ticks() {
        let mut wheel = Wheel::<u32>::new(|_| {});

        wheel.schedule(1, (64 * 64) + 1, SystemTime::now());
        assert_eq!(wheel.next_ticks(), (64 * 64));

        wheel.schedule(1, (64 * 64), SystemTime::now());
        assert_eq!(wheel.next_ticks(), (64 * 64));

        wheel.schedule(1, (64 * 64) - 1, SystemTime::now());
        assert_eq!(wheel.next_ticks(), (64 * (64 - 2)));
    }

    #[test]
    #[ignore] // this test fn will spend 100 seconds
    fn homeless_test() {
        // let mut wheel = Wheel::<String>::new(|_| {});

        // let notice = |e| panic!("notice {}", e);

        // wheel.tick(1, notice);
        // wheel.tick(1, notice);

        // let max_size = 1 << (6 * 6);
        // wheel.schedule("max_offset - 1".into(), max_size - 1);
        // wheel.schedule("max_offset".into(), max_size);
        // wheel.schedule("max_offset + 1".into(), max_size + 1);

        // let empty_times = wheel.ticks + max_size - 2;
        // loop {
        //     wheel.tick(64, notice);
        //     if wheel.ticks >= empty_times - 64 {
        //         break;
        //     }
        // }
        // loop {
        //     wheel.tick(1, notice);
        //     if wheel.ticks >= empty_times {
        //         break;
        //     }
        // }

        // let (tx, rx) = channel::<()>();
        // wheel.tick(1, |e| {
        //     tx.send(()).unwrap();
        //     assert_eq!(e, "max_offset - 1");
        // });
        // wheel.tick(1, |e| {
        //     tx.send(()).unwrap();
        //     assert_eq!(e, "max_offset");
        // });
        // wheel.tick(1, |e| {
        //     tx.send(()).unwrap();
        //     assert_eq!(e, "max_offset + 1");
        // });
        // assert!(rx.try_recv().is_ok());
        // assert!(rx.try_recv().is_ok());
        // assert!(rx.try_recv().is_ok());
    }
}

