use super::{bucket::Bucket, slot::Content};
use std::{
    fmt::Debug,
    time::{SystemTime, UNIX_EPOCH},
};

const LEVEL_COUNT: usize = 6;

#[derive(Debug)]
pub struct Wheel<T> {
    buckets: [Bucket<T>; LEVEL_COUNT],
    pub(crate) tick_times: u64,
    homeless_item: Vec<T>,
}

impl<T: Debug> Wheel<T> {
    pub(crate) fn new() -> Self {
        let mut buckets = Vec::with_capacity(LEVEL_COUNT);

        for level in 0..LEVEL_COUNT {
            buckets.push(Bucket::new(level as u32));
        }

        Wheel {
            buckets: buckets.try_into().unwrap(),
            tick_times: 0,
            homeless_item: vec![],
        }
    }

    // let five_seconds = Duration::new(5, 0);
    pub(crate) fn schedule(&mut self, item: T, tick_times: u128) {
        if let Some(level) = to_level(tick_times) {
            let entity = Content {
                data: item,
                tick_times: tick_times as u64,
            };
            self.buckets[level].add(entity, tick_times as u64);
        } else {
            self.homeless_item.push(item);
        }
    }

    pub(crate) fn tick<F>(&mut self, times: u32, notice: F) -> u32
    where
        F: Fn(T),
    {
        let (result, tick_times, need_next_tick) = self.buckets[0].tick(times);
        if let Some(entities) = result {
            for entity in entities {
                notice(entity.data)
            }
        }

        self.tick_times += tick_times as u64;

        if !need_next_tick {
            return tick_times;
        }

        for level in 1..6 {
            let (result, _, nexneed_next_tickt) = self.buckets[level].tick(1);
            if let Some(entities) = result {
                for entity in entities {
                    if entity.tick_times <= self.tick_times as u64 {
                        // notice
                        notice(entity.data);
                    } else {
                        // add to wheel agin
                        let new_tick_times = entity.tick_times - self.tick_times;

                        let level = to_level(new_tick_times as u128);
                        self.buckets[level.unwrap()].add(entity, new_tick_times);
                    }
                }
            }

            if !nexneed_next_tickt {
                return tick_times;
            }
        }

        tick_times
    }

    pub(crate) fn next_tick_times(&self) -> u32 {
        self.buckets[0].next_tick_times()
    }
}

fn to_level(times: u128) -> Option<usize> {
    const SIZE_OF_LEVEL_0: u128 = 1 << (6 * 1);
    const SIZE_OF_LEVEL_1: u128 = 1 << (6 * 2);
    const SIZE_OF_LEVEL_2: u128 = 1 << (6 * 3);
    const SIZE_OF_LEVEL_3: u128 = 1 << (6 * 4);
    const SIZE_OF_LEVEL_4: u128 = 1 << (6 * 5);
    const SIZE_OF_LEVEL_5: u128 = 1 << (6 * 6);
    match times {
        t if t < SIZE_OF_LEVEL_0 => Some(0),
        t if t < SIZE_OF_LEVEL_1 => Some(1),
        t if t < SIZE_OF_LEVEL_2 => Some(2),
        t if t < SIZE_OF_LEVEL_3 => Some(3),
        t if t < SIZE_OF_LEVEL_4 => Some(4),
        t if t < SIZE_OF_LEVEL_5 => Some(5),
        _ => None,
    }
}

fn _current_millis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

mod tests {

    #[test]
    fn new_test_random() {
        use super::*;
        use rand::Rng;
        let mut rng = rand::thread_rng();

        const MAX_SIZE: u64 = 500;

        let mut wheel = Wheel::<String>::new();

        const ITEM_COUNT: u64 = 200;
        for _ in 0..ITEM_COUNT {
            let millis: u64 = rng.gen_range(1..=MAX_SIZE);
            let _result = wheel.schedule(millis.to_string(), millis as u128);
            // assert!(result.is_ok());
        }

        let mut tick_count = 0u64;
        for _ in 0..=MAX_SIZE {
            tick_count += 1;
            wheel.tick(1, |item| {
                let item_tick: u64 = item.parse().unwrap();
                println!(" - got {:?} ", item);
                assert_eq!(item_tick, tick_count);
            });
        }

        // assert_eq!(real_item_count, ITEM_COUNT);
    }
}
