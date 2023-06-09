#[derive(Debug)]
pub struct Item {
    pub data: String,
    pub(crate) at_tick_times: u64,
}

#[derive(Debug)]
pub(crate) struct Slot {
    pub(crate) items: Option<Vec<Item>>,
}

impl Slot {
    pub(crate) fn new() -> Self {
        Slot { items: None }
    }
    pub(crate) fn push(&mut self, item: Item) {
        if self.items.is_none() {
            self.items = Option::Some(vec![]);
        }

        self.items.as_mut().unwrap().push(item);
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn size_of_item() {
        use super::Item;

        use std::mem::{align_of, size_of};

        assert_eq!(size_of::<Item>(), 32);
        assert_eq!(align_of::<Item>(), 8);
    }
}
