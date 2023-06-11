use super::{bucket::Bucket, slot::Content};
use crate::{TimerError, TimerResult};
use std::time::{SystemTime, UNIX_EPOCH};

const LEVEL_COUNT: u32 = 6;

pub enum SlotSize {
    // 2
    Mini,
    // 32
    Small,
    // 64
    Normal,
    // 128
    Large,
}

#[derive(Debug)]
pub struct Wheel<T> {
    bucket: Bucket<T>,
    _slot_count: u64,
    pub(crate) tick_times: u64,
    capacity: u64,
}

impl<T> Default for Wheel<T> {
    fn default() -> Self {
        Wheel::new(SlotSize::Normal)
    }
}

impl<T> Wheel<T> {
    pub(crate) fn new(slot_count: SlotSize) -> Self {
        let slot_count = match slot_count {
            SlotSize::Mini => 4u64,
            SlotSize::Small => 32u64,
            SlotSize::Normal => 64u64,
            SlotSize::Large => 128u64,
        };

        let mut bucket = Option::None;
        let mut capacity = 0;

        for level in 0..LEVEL_COUNT {
            let raw_bucket = Bucket::new(slot_count, LEVEL_COUNT - level, bucket);
            capacity += raw_bucket.capacity;
            bucket = Option::Some(Box::new(raw_bucket));
        }

        Wheel {
            bucket: *bucket.unwrap(),
            _slot_count: slot_count,
            tick_times: 0,
            capacity,
        }
    }

    // let five_seconds = Duration::new(5, 0);
    pub(crate) fn schedule(&mut self, content: T, tick_times: u128) -> TimerResult<()> {
        if tick_times > self.capacity as u128 {
            return Result::Err(TimerError::OutOfRangeError);
        }

        let tick_times = tick_times as u64;

        let item = Content {
            data: content,
            at_tick_times: self.tick_times + tick_times,
        };

        self.bucket.add(item, tick_times as u64)
    }

    pub(crate) fn tick(&mut self) -> Option<Vec<Content<T>>> {
        self.tick_times += 1;
        self.bucket.tick()
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
    fn new_test() {
        use super::*;
        let wheel = Wheel::<String>::new(SlotSize::Mini);
        println!("{:?}", wheel);
    }

    #[test]
    fn new_test1() {
        use super::*;
        let mut wheel = Wheel::<String>::new(SlotSize::Mini);
        for millis in 1..100 {
            print!("ms: {}\t", millis);
            let result = wheel.schedule("".to_string(), millis);
            assert!(result.is_ok())
        }
    }

    #[test]
    fn new_test_all1() {
        use super::*;
        let mut wheel = Wheel::<String>::new(SlotSize::Mini);
        let result = wheel.schedule("242".to_string(), 242);
        assert!(result.is_ok());
        for i in 1..300 {
            println!("tick:{}", i);
            let _ = wheel.tick();
        }
    }

    #[test]
    fn new_test_random() {
        use super::*;
        use rand::Rng;
        let mut rng = rand::thread_rng();

        const MAX_SIZE: u64 = 500;

        let mut wheel = Wheel::<String>::new(SlotSize::Mini);

        const ITEM_COUNT: u64 = 200;
        for _ in 0..ITEM_COUNT {
            let millis: u64 = rng.gen_range(1..=MAX_SIZE);
            let result = wheel.schedule(millis.to_string(), millis as u128);
            assert!(result.is_ok());
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
