use std::fmt::Debug;

use crate::TimerResult;

use super::slot::{Content, Slot};

use crate::TimerError::{InternalError, OutOfRangeError};

/// Number of slots in a bucket
const SLOT_NUM: u32 = 64;

#[derive(Debug)]
pub(crate) struct Bucket<T> {
    /// Tracking which slots currently contain entries.
    occupied: u64,
    /// Current slot index
    cursor: u32,
    // Slots
    slots: [Slot<T>; SLOT_NUM as usize],
    /// Store the slot which is exceeds the capacity
    homeless_slot: Slot<T>,
    /// Capacity
    capacity: u64,

    /// Tick times
    tick_times: u64,

    next: Option<Box<Bucket<T>>>,

    step_size_in_bits: u32,

    _level: u32,
}

impl<T: Debug> Bucket<T> {
    pub fn new(level: u32, next: Option<Box<Bucket<T>>>) -> Self {
        let power = SLOT_NUM.ilog2();

        let capacity = SLOT_NUM.pow(level);

        let step_size_in_bits = power * (level - 1);

        let mut slots = Vec::<Slot<T>>::with_capacity(SLOT_NUM as usize);
        for _ in 0..SLOT_NUM {
            slots.push(Slot::<T>::new());
        }

        Bucket {
            occupied: 0,
            cursor: 0,
            slots: slots.try_into().unwrap(),

            homeless_slot: Slot::new(),

            capacity: 2u64.pow(power * level) - 1,
            tick_times: 0,

            next,
            step_size_in_bits,
            _level: level,
        }
    }

    pub fn add(&mut self, item: Content<T>, tick_times: u64) {
        debug_assert!(tick_times > 0, "tick times is not allow zero");

        if tick_times >= self.capacity {
            // over this bucket capacity, try to add next level bucket
            return match self.next.as_mut() {
                Some(bucket) => bucket.add(item, tick_times - self.capacity),
                None => self.homeless_slot.push(item),
            };
        }

        // TODO: there will be panic, tick_times小于1了
        let slot_index = (tick_times >> self.step_size_in_bits) as u32;

        debug_assert!(slot_index > 0, "slot index is not allow zero");

        // mark there has entity
        self.occupied |= 1 << (slot_index - 1);

        let slot_index_from_cur = (slot_index + self.cursor) % SLOT_NUM;

        // println!(
        //     " level:{}, index: {},  tick_times:{}, capacity:{},self.step_size_in_bits:{}",
        //     self.level, slot_index, tick_times, self.capacity, self.step_size_in_bits
        // );

        println!(
            " {} is store in [level:{}, index: {}, index from cur:{}]",
            tick_times, self._level, slot_index, slot_index_from_cur
        );

        self.slots[slot_index_from_cur as usize].push(item);
    }

    pub fn tick(&mut self) -> Option<Vec<Content<T>>> {
        self.tick_times += 1;
        self.cursor = (self.tick_times % SLOT_NUM as u64) as u32;

        // there is no entiry
        self.occupied = self.occupied >> 1;

        self.slots[self.cursor as usize].items.take()
    }

    fn tick_next_bucket(&mut self) -> Option<Vec<Content<T>>> {
        if self.next.is_none() {
            // TODO: handle homeless
            //       new thread
        } else if self.cursor == 0 {
            let bucket = self.next.as_mut().unwrap();
            let mut result = bucket.tick();
            let mut next_result = bucket.tick_next_bucket();

            return if result.is_none() {
                next_result
            } else if next_result.is_none() {
                result
            } else {
                let result_vec = result.as_mut().unwrap();
                let next_result_vec = next_result.as_mut().unwrap();

                result_vec.append(next_result_vec);

                result
            };
        }

        None
    }
}

mod test {
    #[test]
    fn test() {
        for level in 1..6 {
            let bucket = super::Bucket::<String>::new(level, None);
            println!("{}, bucket: {:?}", level, bucket);
        }
    }
}
