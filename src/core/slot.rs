use std::time::SystemTime;

#[derive(Debug, PartialEq, Eq)]
pub struct Entity<T> {
    pub data: T,
    pub(crate) tick_times: u64,
    pub(crate) when: SystemTime,
    pub(crate) ticks: u64,
    pub(crate) offset: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Slot<T> {
    pub(crate) items: Option<Vec<Entity<T>>>,
}

impl<T> Slot<T> {
    pub(crate) fn new() -> Self {
        Slot { items: None }
    }
    pub(crate) fn push(&mut self, item: Entity<T>) {
        self.items.get_or_insert_with(Vec::new).push(item);

        // if self.items.is_none() {
        //     self.items = Option::Some(vec![]);
        // }

        // self.items.as_mut().unwrap().push(item);
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn size_of_item() {
        use super::Entity;

        use std::mem::{align_of, size_of};
        assert_eq!(size_of::<Entity<String>>(), 32);
        assert_eq!(align_of::<Entity<String>>(), 8);
    }
}
