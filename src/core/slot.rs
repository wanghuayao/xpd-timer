#[derive(Debug)]
pub struct Item {
    pub data: String,
    // pub when: u128,
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
