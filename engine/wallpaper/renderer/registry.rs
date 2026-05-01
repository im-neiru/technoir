use core::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};

use indexmap::IndexMap;

pub struct Registry<Key, Index, Value>
where
    Key: Eq + Hash,
    Index: Eq + Hash + Copy + Clone + From<usize> + Debug,
{
    items: IndexMap<Key, Item<Value>>,
    _marker: PhantomData<Index>,
}

struct Item<Value> {
    value: Value,
    ref_count: usize,
}

impl<Key, Index, Value> Registry<Key, Index, Value>
where
    Key: Eq + Hash,
    Index: Eq + Hash + Copy + Clone + From<usize> + Debug,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            items: IndexMap::new(),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn get_or_insert<E>(
        &mut self,
        key: Key,
        create: impl FnOnce() -> Result<Value, E>,
    ) -> Result<(Index, &Value), E> {
        if let Some(index) = self.items.get_index_of(&key) {
            let item = &mut self.items[index];
            item.ref_count += 1;
            return Ok((Index::from(index), &item.value));
        }

        let value = create()?;
        let (index, _) = self.items.insert_full(
            key,
            Item {
                value,
                ref_count: 1,
            },
        );

        Ok((Index::from(index), &self.items[index].value))
    }

    #[inline]
    pub fn get_by_index(&self, index: usize) -> Option<&Value> {
        let (_, item) = self.items.get_index(index)?;

        Some(&item.value)
    }

    #[inline]
    pub(super) fn release_by_index(&mut self, index: usize) -> Option<Value> {
        let (_, item) = self.items.get_index_mut(index)?;

        item.ref_count -= 1;

        if item.ref_count == 0 {
            let (_, item) = self.items.shift_remove_index(index).unwrap();
            Some(item.value)
        } else {
            None
        }
    }

    pub(super) fn len(&self) -> usize {
        self.items.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&Key, &Value)> {
        self.items.iter().map(|(k, i)| (k, &i.value))
    }
}

impl<Key, Index, Value> Default for Registry<Key, Index, Value>
where
    Key: Eq + Hash,
    Index: Eq + Hash + Copy + Clone + From<usize> + Display + Debug,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
