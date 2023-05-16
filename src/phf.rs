use std::{fmt, iter::FusedIterator};

use phf_shared::HashKey;

pub struct CharMap<V: 'static> {
    #[doc(hidden)]
    pub range: std::ops::RangeInclusive<char>,
    #[doc(hidden)]
    pub key: HashKey,
    #[doc(hidden)]
    pub disps: &'static [(u32, u32)],
    #[doc(hidden)]
    pub entries: &'static [(char, V)],
}

impl<V: 'static> CharMap<V> {
    #[inline]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn contains_key(&self, key: char) -> bool {
        self.get(key).is_some()
    }

    pub fn get(&self, key: char) -> Option<&V> {
        self.get_entry(key).map(|e| e.1)
    }

    pub fn get_entry(&self, key: char) -> Option<(char, &V)> {
        if !self.range.contains(&key) {
            return None;
        }

        let hashes = phf_shared::hash(&key, &self.key);
        let index = phf_shared::get_index(&hashes, self.disps, self.entries.len());
        let entry = &self.entries[index as usize];
        if key == entry.0 {
            Some((entry.0, &entry.1))
        } else {
            None
        }
    }

    #[inline]
    pub fn entries(&self) -> Entries<V> {
        Entries {
            iter: self.entries.iter(),
        }
    }

    #[inline]
    pub fn keys(&self) -> Keys<V> {
        Keys {
            iter: self.entries.iter(),
        }
    }

    #[inline]
    pub fn values(&self) -> Values<V> {
        Values {
            iter: self.entries.iter(),
        }
    }
}

pub struct Entries<'a, V> {
    iter: std::slice::Iter<'a, (char, V)>,
}

impl<'a, V> Clone for Entries<'a, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, V> Iterator for Entries<'a, V> {
    type Item = (char, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&(c, ref v)| (c, v))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, V> DoubleEndedIterator for Entries<'a, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|&(c, ref v)| (c, v))
    }
}

impl<'a, V> ExactSizeIterator for Entries<'a, V> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, V> FusedIterator for Entries<'a, V> {}

impl<'a, V: fmt::Debug> fmt::Debug for Entries<'a, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

pub struct Keys<'a, V> {
    iter: std::slice::Iter<'a, (char, V)>,
}

impl<'a, V> Clone for Keys<'a, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, V> Iterator for Keys<'a, V> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&(c, _)| c)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, V> DoubleEndedIterator for Keys<'a, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|&(c, _)| c)
    }
}

impl<'a, V> ExactSizeIterator for Keys<'a, V> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, V> FusedIterator for Keys<'a, V> {}

impl<'a, V: fmt::Debug> fmt::Debug for Keys<'a, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

pub struct Values<'a, V> {
    iter: std::slice::Iter<'a, (char, V)>,
}

impl<'a, V> Clone for Values<'a, V> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, V> Iterator for Values<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, V> DoubleEndedIterator for Values<'a, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(_, v)| v)
    }
}

impl<'a, V> ExactSizeIterator for Values<'a, V> {
    #[inline]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, V> FusedIterator for Values<'a, V> {}

impl<'a, V: fmt::Debug> fmt::Debug for Values<'a, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
