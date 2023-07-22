use super::{bucket::Bucket, slot::Entity};
use std::{
    fmt::Debug,
    time::{SystemTime, UNIX_EPOCH},
};

const LEVEL_COUNT: usize = 6;

#[derive(Debug)]
pub struct Wheel<T> {
    buckets: [Bucket<T>; LEVEL_COUNT],
    pub(crate) ticks: u64,
    homeless: Option<Vec<Entity<T>>>,
}

impl<T: Debug> Wheel<T> {
    pub(crate) fn new() -> Self {
        let mut buckets = Vec::with_capacity(LEVEL_COUNT);

        for level in 0..LEVEL_COUNT {
            buckets.push(Bucket::new(level as u32));
        }

        Wheel {
            buckets: buckets.try_into().unwrap(),
            ticks: 0,
            homeless: None,
        }
    }

    pub(crate) fn schedule(&mut self, entity: T, offset: u64) {
        let entity = Entity {
            data: entity,
            tick_times: offset + self.ticks,
        };

        if let Some(level) = to_level(offset) {
            self.buckets[level].add(entity, offset as u64);
        } else {
            if self.homeless.is_none() {
                self.homeless = Some(vec![])
            }
            self.homeless.as_mut().unwrap().push(entity);
        }
    }

    pub(crate) fn tick<F>(&mut self, times: u32, notice: F)
    where
        F: Fn(T),
    {
        let (result, real_ticks, is_need_tick_next_level) = self.buckets[0].tick(times);

        if let Some(entities) = result {
            for entity in entities {
                notice(entity.data)
            }
        }

        self.ticks += real_ticks as u64;
        if is_need_tick_next_level {
            self.tick_next_level(&notice);
        }

        if real_ticks < times {
            self.tick(times - real_ticks, notice)
        }
    }

    pub(crate) fn next_tick_times(&self) -> u32 {
        self.buckets[0].next_tick_times()
    }

    fn tick_next_level<F>(&mut self, notice: &F)
    where
        F: Fn(T),
    {
        for level in 1..LEVEL_COUNT {
            let (result, _, is_need_tick_next_level) = self.buckets[level].tick(1);

            if let Some(entities) = result {
                for entity in entities {
                    if entity.tick_times <= self.ticks as u64 {
                        notice(entity.data);
                    } else {
                        // add to wheel agin
                        let offset = entity.tick_times - self.ticks;
                        let level = to_level(offset);

                        self.buckets[level.unwrap()].add(entity, offset);
                    }
                }
            }

            if level == LEVEL_COUNT - 1 && self.homeless.is_some() {
                // tick to last level, rerange homeless
                let entitys = self.homeless.take().unwrap();
                for entity in entitys {
                    // add to wheel agin
                    let offset = entity.tick_times - self.ticks;
                    let level = to_level(offset);
                    self.buckets[level.unwrap()].add(entity, offset);
                }
            }

            if !is_need_tick_next_level {
                break;
            }
        }
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
}

fn _current_millis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

mod tests {
    use std::sync::mpsc::channel;

    #[test]
    fn new_test_random() {
        use super::*;
        use rand::Rng;
        let mut rng = rand::thread_rng();

        const MAX_SIZE: u64 = 500;

        let mut wheel = Wheel::<String>::new();

        const ITEM_COUNT: u64 = 200;
        for _ in 0..ITEM_COUNT {
            let offset: u64 = rng.gen_range(1..=MAX_SIZE);
            let _result = wheel.schedule(offset.to_string(), offset);
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

    // this case will run 100s
    // #[test]
    #[allow(dead_code)]
    fn homeless_test() {
        use super::*;
        let mut wheel = Wheel::<String>::new();

        let notice = |e| panic!("notice {}", e);

        wheel.tick(1, notice);
        wheel.tick(1, notice);

        let max_size = 1 << (6 * 6);
        wheel.schedule("max_offset - 1".into(), max_size - 1);
        wheel.schedule("max_offset".into(), max_size);
        wheel.schedule("max_offset + 1".into(), max_size + 1);

        let empty_times = wheel.ticks + max_size - 2;
        loop {
            wheel.tick(64, notice);
            if wheel.ticks >= empty_times - 64 {
                break;
            }
        }
        loop {
            wheel.tick(1, notice);
            if wheel.ticks >= empty_times {
                break;
            }
        }

        let (tx, rx) = channel::<()>();
        wheel.tick(1, |e| {
            tx.send(()).unwrap();
            assert_eq!(e, "max_offset - 1");
        });
        wheel.tick(1, |e| {
            tx.send(()).unwrap();
            assert_eq!(e, "max_offset");
        });
        wheel.tick(1, |e| {
            tx.send(()).unwrap();
            assert_eq!(e, "max_offset + 1");
        });
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_ok());
    }
}
