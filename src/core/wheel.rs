use super::{bucket::Bucket, slot::Entity};
use std::{
    convert::TryInto,
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
        let buckets = (0..LEVEL_COUNT)
            .map(|level| Bucket::new(level as u32))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Wheel {
            buckets,
            ticks: 0,
            homeless: None,
        }
    }

    pub(crate) fn schedule(&mut self, entity: T, offset: u64) {
        let entity = Entity {
            data: entity,
            tick_times: offset + self.ticks,
        };

        match to_level(offset) {
            Some(level) => self.buckets[level].add(entity, offset as u64),
            _ => self.homeless.get_or_insert_with(Vec::new).push(entity),
        }
    }

    pub(crate) fn tick<F>(&mut self, times: u32, notice: F)
    where
        F: Fn(T),
    {
        let (result, real_ticks, is_need_tick_next_level) = self.buckets[0].tick(times);

        self.ticks += real_ticks as u64;

        if let Some(entities) = result {
            // entities.into_iter().for_each(|entity| {
            //     if self.ticks != entity.tick_times {
            //         dbg!(self.ticks, &entity);
            //         debug_assert_eq!(self.ticks, entity.tick_times);
            //     }
            //     notice(entity.data);
            // });
            for entity in entities {
                notice(entity.data)
            }
        }

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
                        dbg!(self.ticks, &entity);
                        notice(entity.data);
                    } else {
                        // add to wheel again
                        let offset = entity.tick_times - self.ticks;
                        let level = to_level(offset);

                        self.buckets[level.unwrap()].add(entity, offset);
                    }
                }
            }

            if level == LEVEL_COUNT - 1 && self.homeless.is_some() {
                // tick to last level, rearrange homeless
                let entities = self.homeless.take().unwrap();
                for entity in entities {
                    // add to wheel again
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
    use rand::Rng;
    use std::sync::mpsc::channel;

    #[test]
    fn new_test_random() {
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

    #[test]
    #[ignore] // this test fn will spend 100 seconds
    fn homeless_test() {
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
