use super::slot::{Entity, Slot};
use std::convert::TryInto;
use std::fmt::Debug;

/// Number of slots in a bucket
pub(super) const SLOT_NUM: u32 = 64;

#[derive(Debug)]
pub(crate) struct Bucket<T> {
    /// Tracking which slots currently contain entries.
    occupied: u64,
    /// Current slot index
    cursor: u32,
    /// Slots
    slots: [Slot<T>; SLOT_NUM as usize],

    /// Tick times
    // tick_times: u64,
    step_size_in_bits: u32,

    _level: u32,
}

impl<T: Debug> Bucket<T> {
    /// New bucket `level` is from 0.
    pub fn new(level: u32) -> Self {
        let power = SLOT_NUM.ilog2();

        let step_size_in_bits = power * level;

        let slots = (0..SLOT_NUM)
            .map(|_| Slot::<T>::new())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Bucket {
            occupied: 0,
            cursor: 0,
            slots,

            // tick_times: 0,
            step_size_in_bits,
            _level: level,
        }
    }

    pub fn add(&mut self, entity: Entity<T>, offset: u64) {
        debug_assert!(offset > 0, "tick times is not allow zero");

        // TODO: there will be panic, tick_times小于1了
        let slot_index = (offset >> self.step_size_in_bits) as u32;

        debug_assert!(slot_index > 0, "slot index is not allow zero");
        debug_assert!(slot_index < 64, "slot index is overflow");

        // mark there has entity
        self.occupied |= 1 << (slot_index - 1);

        let slot_index_from_cur = (slot_index + self.cursor) % SLOT_NUM;

        self.slots[slot_index_from_cur as usize].push(entity);
    }

    /// tick
    pub fn tick(&mut self, times: u32) -> (Option<Vec<Entity<T>>>, u32, bool) {
        // distance from  start
        let tick_times = times.min(SLOT_NUM - self.cursor);

        let empty_times = tick_times.min(self.occupied.trailing_zeros());

        if empty_times > 0 {
            // there no Entity, only move cursor
            // self.tick_times += empty_times as u64;
            self.cursor = ((self.cursor + empty_times) % SLOT_NUM) as u32;
            self.occupied = if empty_times < SLOT_NUM {
                self.occupied >> empty_times
            } else {
                0
            };

            let is_back = self.cursor == 0;
            if times == empty_times || is_back {
                // return parent for tick next level
                return (None, empty_times, is_back);
            }
        }

        // tick one time
        // self.tick_times += 1;
        self.cursor = ((self.cursor + 1) % SLOT_NUM) as u32;
        let is_empty = (self.occupied & 1) == 0;
        // there is no entiry
        self.occupied = self.occupied >> 1;

        let result = if is_empty {
            None
        } else {
            self.slots[self.cursor as usize].items.take()
        };

        // result, tick times, need tick next
        (result, empty_times + 1, self.cursor == 0)
    }

    pub(crate) fn next_tick_times(&self) -> u32 {
        let distance_to_zero = SLOT_NUM - self.cursor;

        let next_entity_pos = self.occupied.trailing_zeros() + 1;

        distance_to_zero.min(next_entity_pos)
    }
}

// Test
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! content {
        ($item:expr) => {
            Entity {
                data: $item,
                tick_times: $item,
            }
        };
        ($item:expr, $times: expr) => {
            Entity {
                data: $item,
                tick_times: $times,
            }
        };
    }

    #[test]
    fn test_new() {
        let bucket = Bucket::<i64>::new(0);
        assert_eq!(bucket.occupied, 0);
        assert_eq!(bucket.cursor, 0);
        // assert_eq!(bucket.tick_times, 0);
        assert_eq!(bucket.step_size_in_bits, 0);
        assert_eq!(bucket._level, 0);
        assert_eq!(bucket.slots.len(), 64);
    }

    #[test]
    fn test_add() {
        // level 0
        let mut bucket = Bucket::<u64>::new(0);
        bucket.add(content!(63), 63);
        assert_eq!(bucket.occupied, 1u64 << (63 - 1));

        bucket.add(content!(163), 63);
        assert_eq!(bucket.occupied, 1u64 << (63 - 1));

        bucket.add(content!(8), 8);
        assert_eq!(bucket.occupied, 1u64 << (63 - 1) | 1u64 << (8 - 1));

        let items = bucket.slots[63].items.take().unwrap();
        assert_eq!(items.len(), 2);

        let items = bucket.slots[8].items.take().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].data, 8);

        // level 1
        let mut bucket2 = Bucket::<i64>::new(1);
        bucket2.add(content!(64), 64);
        bucket2.add(content!(65), 65);
        assert_eq!(bucket2.occupied, 1u64 << (1 - 1));

        bucket2.add(content!(128), 128);
        assert_eq!(bucket2.occupied, 1u64 << (1 - 1) | 1u64 << (2 - 1));

        let items = bucket2.slots[1].items.take().unwrap();
        assert_eq!(items.len(), 2);

        let items = bucket2.slots[2].items.take().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].data, 128);
    }

    #[test]
    fn test_tick() {
        let mut bucket = Bucket::<u64>::new(0);
        bucket.add(content!(1), 1);
        bucket.add(content!(5), 5);
        assert_eq!(bucket.occupied, 0b0001_0001);

        let (result, tick_times, need_tick_next) = bucket.tick(1);
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data, 1);
        assert_eq!(tick_times, 1);
        assert_eq!(need_tick_next, false);
        assert_eq!(bucket.cursor, 1);
        assert_eq!(bucket.occupied, 0b0000_1000);

        let (result, tick_times, need_tick_next) = bucket.tick(4);
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data, 5);
        assert_eq!(tick_times, 4);
        assert_eq!(need_tick_next, false);
        assert_eq!(bucket.cursor, 5);
        assert_eq!(bucket.occupied, 0b0000_0000);

        bucket.add(content!(105), 5);
        assert_eq!(bucket.occupied, 0b0001_0000);
        let (result, tick_times, need_tick_next) = bucket.tick(4);
        assert_eq!(result, None);
        assert_eq!(tick_times, 4);
        assert_eq!(need_tick_next, false);
        assert_eq!(bucket.cursor, 9);
        assert_eq!(bucket.occupied, 0b0000_0001);

        let (result, tick_times, need_tick_next) = bucket.tick(4);
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data, 105);
        assert_eq!(tick_times, 1);
        assert_eq!(need_tick_next, false);
        assert_eq!(bucket.cursor, 10);
        assert_eq!(bucket.occupied, 0b0000_0000);

        let (result, tick_times, need_tick_next) = bucket.tick(100);
        assert_eq!(result, None);
        assert_eq!(tick_times, 54);
        assert_eq!(need_tick_next, true);
        assert_eq!(bucket.cursor, 0);
        assert_eq!(bucket.occupied, 0b0000_0000);
    }

    #[test]
    fn test_next_tick_interval() {
        let mut bucket = Bucket::<u64>::new(0);
        assert_eq!(bucket.next_tick_times(), SLOT_NUM);

        bucket.add(content!(1), 1);
        assert_eq!(bucket.next_tick_times(), 1);

        bucket.tick(1);
        assert_eq!(bucket.next_tick_times(), SLOT_NUM - 1);

        bucket.tick(10);
        assert_eq!(bucket.next_tick_times(), SLOT_NUM - 1 - 10);

        bucket.add(content!((SLOT_NUM - 1) as u64), (SLOT_NUM - 1) as u64);

        assert_eq!(bucket.next_tick_times(), SLOT_NUM - 1 - 10);
    }
}
