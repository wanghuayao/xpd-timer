use std::fmt::Debug;

use super::slot::{Content, Slot};

/// Number of slots in a bucket
const SLOT_NUM: u32 = 64;

#[derive(Debug)]
pub(crate) struct Bucket<T> {
    /// Tracking which slots currently contain entries.
    occupied: u64,
    /// Current slot index
    cursor: u32,
    /// Slots
    slots: [Slot<T>; SLOT_NUM as usize],

    /// Tick times
    tick_times: u64,

    step_size_in_bits: u32,

    _level: u32,
}

impl<T: Debug> Bucket<T> {
    pub fn new(level: u32) -> Self {
        let power = SLOT_NUM.ilog2();

        let step_size_in_bits = power * (level - 1);

        let mut slots = Vec::<Slot<T>>::with_capacity(SLOT_NUM as usize);
        for _ in 0..SLOT_NUM {
            slots.push(Slot::<T>::new());
        }

        Bucket {
            occupied: 0,
            cursor: 0,
            slots: slots.try_into().unwrap(),

            tick_times: 0,

            step_size_in_bits,
            _level: level,
        }
    }

    pub fn add(&mut self, data: T, tick_times: u64) {
        debug_assert!(tick_times > 0, "tick times is not allow zero");

        // TODO: there will be panic, tick_times小于1了
        let slot_index = (tick_times >> self.step_size_in_bits) as u32;

        debug_assert!(slot_index > 0, "slot index is not allow zero");

        // mark there has entity
        self.occupied |= 1 << (slot_index - 1);

        let slot_index_from_cur = (slot_index + self.cursor) % SLOT_NUM;

        println!(
            " {} is store in [level:{}, index: {}, index from cur:{}]",
            tick_times, self._level, slot_index, slot_index_from_cur
        );

        self.slots[slot_index_from_cur as usize].push(Content {
            data,
            at_tick_times: self.tick_times + tick_times as u64,
        });
    }

    /// tick
    /// return
    pub fn tick(&mut self) -> (Option<Vec<Content<T>>>, u64, bool) {
        self.tick_times += 1;
        self.cursor = (self.tick_times % SLOT_NUM as u64) as u32;
        let is_empty = (self.occupied & 1) == 1;
        // there is no entiry
        self.occupied = self.occupied >> 1;

        let result = if is_empty {
            None
        } else {
            self.slots[self.cursor as usize].items.take()
        };

        // result, tick times, need tick next
        (result, 1, self.cursor == 0)
    }

    /// tick
    pub fn _tick_all(&mut self, max_times: u64) -> (Option<Vec<Content<T>>>, u64, bool) {
        let safe_tick_times = self.occupied.trailing_zeros() as u64;
        // max value move to zero
        let can_tick_times = (SLOT_NUM - self.cursor) as u64;

        let real_tick_times = can_tick_times.min(max_times);

        let empty_times = safe_tick_times.min(real_tick_times);

        self.tick_times += empty_times;
        self.cursor = (self.tick_times % SLOT_NUM as u64) as u32;
        self.occupied = self.occupied >> empty_times;

        let result = if real_tick_times > safe_tick_times {
            let mut contents = Vec::<Content<T>>::new();
            for _i in 0..=(real_tick_times - safe_tick_times) {
                self.tick_times += 1;
                self.cursor = (self.tick_times % SLOT_NUM as u64) as u32;
                self.occupied = self.occupied >> 1;
                let current_result = self.slots[self.cursor as usize].items.take();
                if let Some(mut items) = current_result {
                    // for item in items {
                    //     contents.push(item);
                    // }
                    contents.append(&mut items)
                }
            }
            Some(contents)
        } else {
            None
        };

        // result, tick times, need tick next
        (result, real_tick_times, self.cursor == 0)
    }
}

mod test {
    #[test]
    fn test() {
        // use std::sync::Arc;
        // let mut bucket2 = Arc::new(super::Bucket::<String>::new(2, None));
        // let mut bucket = super::Bucket::<String>::new(1, Some(bucket2.clone()));

        // macro_rules! content {
        //     ($x:expr,$times:expr) => {
        //         $crate::core::slot::Content {
        //             data: $x.to_string(),
        //             at_tick_times: $times,
        //         }
        //     };
        // }

        // bucket.add(content! { "1",63}, 63);
        // bucket.add(content! { "2",64}, 64);

        // assert_eq!(
        //     bucket.slots[63].items.take().unwrap()[0].data,
        //     "1".to_string()
        // );

        // assert_eq!(
        //     bucket2.slots[1].items.take().unwrap()[0].data,
        //     "2".to_string()
        // );
    }
}
