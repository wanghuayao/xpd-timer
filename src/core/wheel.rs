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

impl<T: Debug> Default for Wheel<T> {
    fn default() -> Self {
        Wheel::new()
    }
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
            self.buckets[level].add(item, tick_times as u64);
        } else {
            self.homeless_item.push(item);
        }
    }

    pub(crate) fn tick(&mut self) -> Option<Vec<Content<T>>> {
        for bucket in self.buckets.iter_mut() {
            let (result, tick_times, next) = bucket.tick();
            if let Some(_item) = result {
                // TODO, notice
            }
            self.tick_times += tick_times;
            if !next {
                break;
            }
        }

        // TODO will delete
        None
    }
}

fn to_level(times: u128) -> Option<usize> {
    const SIZE_OF_LEVEL_0: u128 = 2 << (6 * 1);
    const SIZE_OF_LEVEL_1: u128 = 2 << (6 * 2);
    const SIZE_OF_LEVEL_2: u128 = 2 << (6 * 3);
    const SIZE_OF_LEVEL_3: u128 = 2 << (6 * 4);
    const SIZE_OF_LEVEL_4: u128 = 2 << (6 * 5);
    const SIZE_OF_LEVEL_5: u128 = 2 << (6 * 6);
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

        let mut real_item_count = 0u64;
        let mut tick_count = 0u64;
        for _ in 0..=MAX_SIZE {
            tick_count += 1;
            println!("tick: {}", tick_count);

            if let Some(items) = wheel.tick() {
                for item in items {
                    real_item_count += 1;
                    let item_tick: u64 = item.data.parse().unwrap();
                    println!(" - got {:?} ", item);
                    assert_eq!(item_tick, tick_count);
                }
            }
        }

        assert_eq!(real_item_count, ITEM_COUNT);
    }
}
