pub trait IterHelper<Item> {
    fn find_map<B, P>(&mut self, mut predicate: P) -> Option<B> where
        Self: Sized,
        P: FnMut(Item) -> Option<B>;
}

impl<T, Item> IterHelper<Item> for T where T: Iterator<Item=Item> {
    fn find_map<B, P>(&mut self, mut predicate: P) -> Option<B> where
        Self: Sized,
        P: FnMut(Item) -> Option<B>
    {
        for x in self.by_ref() {
            if let Some(y) = predicate(x) {
                return Some(y);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_map() {
        assert_eq!((0u16 .. 100).rev().find_map(|i| i.checked_add(65530)), Some(5));
    }
}

