#[derive(Debug, PartialEq, Eq)]
pub struct Content<T> {
    pub data: T,
    pub(crate) at_tick_times: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Slot<T> {
    pub(crate) items: Option<Vec<Content<T>>>,
}

impl<T> Slot<T> {
    pub(crate) fn new() -> Self {
        Slot { items: None }
    }
    pub(crate) fn push(&mut self, item: Content<T>) {
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
        use super::Content;

        use std::mem::{align_of, size_of};
        assert_eq!(size_of::<Content<String>>(), 32);
        assert_eq!(align_of::<Content<String>>(), 8);
    }
}
