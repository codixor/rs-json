use std::slice::Iter;

#[derive(Debug, Clone)]
pub(crate) enum Container<K, V> {
    Array(Vec<V>),
    Map(Vec<(K, V)>),
}

pub(crate) enum IterArray<'a, V> {
    Empty,
    Vec(Iter<'a, V>),
}

/// Array elements iterator.
pub struct ArrayIterator<'a, V> {
    pub(crate) iter: IterArray<'a, V>,
}

impl<'a, V> Iterator for ArrayIterator<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            IterArray::Empty => None,
            IterArray::Vec(iter) => iter.next(),
        }
    }
}

pub(crate) enum IterMap<'a, K, V> {
    Empty,
    Vec(Iter<'a, (K, V)>),
}

/// Object entries iterator.
pub struct MapIterator<'a, K, V> {
    pub(crate) iter: IterMap<'a, K, V>,
}

impl<'a, K, V> Iterator for MapIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            IterMap::Empty => None,
            IterMap::Vec(iter) => iter.next().map(|(key, value)| (key, value)),
        }
    }
}
