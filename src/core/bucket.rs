use crate::TimerResult;

use super::slot::{Content, Slot};

use crate::TimerError::{InternalError, OutOfRangeError};

#[derive(Debug)]
pub(crate) struct Bucket<T> {
    slots: Vec<Slot<T>>,
    pos: usize,
    step_size_in_bits: u32,
    pub(crate) capacity: u64,
    low_level_capacity: u64,

    tick_times: u64,
    next: Option<Box<Bucket<T>>>,

    _level: u32,
}

impl<T> Bucket<T> {
    pub fn new(slot_count: u64, level: u32, next: Option<Box<Bucket<T>>>) -> Self {
        let capacity = slot_count.pow(level);
        let mut low_level_capacity: u64 = 0;
        for i in 1..level {
            low_level_capacity += slot_count.pow(i);
        }

        let step_size_in_bits = (slot_count as f64).log2() as u32 * (level - 1);

        println!(
            "level:{},capacity:{},step_size_in_bits:{}",
            level, capacity, step_size_in_bits
        );

        let mut slots = vec![];
        for _ in 0..slot_count {
            slots.push(Slot::new());
        }

        Bucket {
            slots,
            next,
            pos: 0,
            step_size_in_bits,
            capacity,
            tick_times: 0,
            low_level_capacity,

            _level: level,
        }
    }

    pub fn add(&mut self, item: Content<T>, tick_times: u64) -> TimerResult<()> {
        if tick_times < 1 {
            return Err(InternalError(String::from("tick times is not allow zero")));
        }

        if tick_times > self.capacity {
            // over this bucket capacity, try to add next level bucket
            return match self.next.as_mut() {
                Some(bucket) => bucket.add(item, tick_times - self.capacity),
                None => Err(OutOfRangeError),
            };
        }

        // TODO: there will be panic, tick_times小于1了
        let slot_index = ((tick_times - 1) >> self.step_size_in_bits) as usize;
        let slot_index = (slot_index + self.pos) % self.slots.len();

        // println!(
        //     " level:{}, index: {},  tick_times:{}, capacity:{},self.step_size_in_bits:{}",
        //     self.level, slot_index, tick_times, self.capacity, self.step_size_in_bits
        // );

        self.slots[slot_index].push(item);

        Ok(())
    }

    pub fn tick(&mut self) -> Option<Vec<Content<T>>> {
        let position = self.pos;
        let result = self.slots[position].items.take();
        self.pos += 1;
        self.tick_times += 1;

        if self.pos == self.slots.len() {
            // reach the max length, reposition the pointer
            self.pos = 0;
            self.relocate_next_bucket();
        }

        result
    }

    fn relocate_next_bucket(&mut self) {
        if self.next.is_none() {
            return;
        }

        let bucket = self.next.as_mut().unwrap();
        let total_tick_time = self.tick_times << self.step_size_in_bits;
        let upper_result = bucket.tick();
        for item in upper_result.unwrap_or_default() {
            let tick_times = item.at_tick_times - total_tick_time - self.low_level_capacity;
            let _ = self.add(item, tick_times);
        }
    }
}

mod test {
    #[test]
    fn test() {
        for level in 1..6 {
            let bucket = super::Bucket::<String>::new(2, level, None);
            println!("{}, bucket: {:?}", level, bucket);
        }
    }
}
