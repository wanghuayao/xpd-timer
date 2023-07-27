use super::slot::{Entity, Slot};
use std::convert::TryInto;
use std::fmt::Debug;

/// power of 2 (2^6 = 64)
pub(super) const SLOT_NUM_POWER_OF_2: u32 = 6;

/// Number of slots in a bucket
pub(super) const SLOT_NUM: u32 = 1 << SLOT_NUM_POWER_OF_2;

const SLOT_NUM_MASK: u32 = SLOT_NUM - 1;

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

        let slot_index_from_cur = (slot_index + self.cursor) & SLOT_NUM_MASK;

        self.slots[slot_index_from_cur as usize].push(entity);
    }

    /// tick (result, next level tick times)
    pub fn tick(&mut self, times: u32) -> (Option<Vec<Entity<T>>>, u32) {
        let mut entities = Option::<Vec<Entity<T>>>::None;

        let next_level_tick_times = (times + self.cursor) >> SLOT_NUM_POWER_OF_2;

        // has some things
        let mut left_times = times;

        if self.occupied > 0 && times > self.occupied.trailing_zeros() {
            // this tick has some entities
            let mut temp_entities = Vec::new();

            while left_times > 0 && self.occupied > 0 {
                let non_empty_index = self.occupied.trailing_zeros();
                let ticks = left_times.min(non_empty_index + 1);

                self.cursor = (self.cursor + ticks) & SLOT_NUM_MASK;

                if let Some(timeout_entities) = self.slots[self.cursor as usize].items.take() {
                    temp_entities.extend(timeout_entities);
                }
                left_times -= ticks;
                self.occupied = self.occupied >> ticks;
            }

            entities = Some(temp_entities);
        }

        self.occupied = self.occupied.checked_shr(left_times).unwrap_or(0);
        self.cursor = (self.cursor + left_times) & SLOT_NUM_MASK;

        return (entities, next_level_tick_times);
    }

    /// get the non-stop ticks
    /// attation: this will return 0
    pub(crate) fn non_stop_ticks(&self) -> u32 {
        let distance_to_zero = SLOT_NUM - self.cursor;
        let next_entity_pos = self.occupied.trailing_zeros();

        (distance_to_zero.min(next_entity_pos)) << self.step_size_in_bits
    }
}

// Test
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    macro_rules! content {
        ($item:expr) => {
            Entity {
                data: $item,
                tick_times: $item,
                when: SystemTime::now(),
                offset: 0,
                ticks: 0,
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

        let (result, next_tick_times) = bucket.tick(1);
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data, 1);
        assert_eq!(next_tick_times, 0);
        assert_eq!(bucket.cursor, 1);
        assert_eq!(bucket.occupied, 0b0000_1000);

        let (result, next_tick_times) = bucket.tick(4);
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data, 5);
        assert_eq!(next_tick_times, 0);
        assert_eq!(bucket.cursor, 5);
        assert_eq!(bucket.occupied, 0b0000_0000);

        bucket.add(content!(105), 5);
        assert_eq!(bucket.occupied, 0b0001_0000);
        let (result, next_tick_times) = bucket.tick(4);
        assert_eq!(result, None);
        assert_eq!(next_tick_times, 0);
        assert_eq!(bucket.cursor, 9);
        assert_eq!(bucket.occupied, 0b0000_0001);

        let (result, next_tick_times) = bucket.tick(4);
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].data, 105);
        assert_eq!(next_tick_times, 0);
        assert_eq!(bucket.cursor, 13);
        assert_eq!(bucket.occupied, 0b0000_0000);

        let (result, next_tick_times) = bucket.tick(100);
        assert_eq!(result, None);
        assert_eq!(next_tick_times, 1);
        assert_eq!(bucket.occupied, 0b0000_0000);
    }

    #[test]
    fn test_non_stop_ticks() {
        let mut bucket = Bucket::<u64>::new(0);
        assert_eq!(bucket.non_stop_ticks(), SLOT_NUM);

        bucket.add(content!(1), 1);
        assert_eq!(bucket.non_stop_ticks(), 0);

        bucket.tick(1);
        assert_eq!(bucket.non_stop_ticks(), SLOT_NUM - 1);

        bucket.tick(10);
        assert_eq!(bucket.non_stop_ticks(), SLOT_NUM - 1 - 10);

        bucket.add(content!((SLOT_NUM - 1) as u64), (SLOT_NUM - 1) as u64);
        assert_eq!(bucket.non_stop_ticks(), SLOT_NUM - 1 - 10);
    }
}
