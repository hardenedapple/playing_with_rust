use std::collections::{ HashMap, HashSet };
use std::rc::Rc;
use std::iter::FromIterator;
use self::order_iter::*;

mod order_iter;

#[cfg(test)]
mod tests;

// TODO
//      More tests
//          Test for the size_hint() of all the different iterators.
//          Add some randomised tests to check the behaviour against that of HashMap<>.
//          Include tests of the ordering in the randomised tests.
//          Randomised tests, checking that the ordering and underlying hash match.
//      Finish Implementation
//          I still have to implement drain(&mut self) and entry(&mut self).
//      Add Documentation
//          I should really document each of the structures, but seeing as the documentation is
//          usually "see HashMap<>, but if this is an iterator it goes in order of inserted keys"
//          it seems a little silly.
//          Documentation for the order_iter module *should* be done.
//      Maybe remove the Debug bound on IntoIterator.
//          I like the extra message that .expect() provides, but if it comes at a cost of being
//          different than the HashMap<> implementation, I'm not sure it's worth it.

// Things I don't really like:
//      I have to specify that K is Eq and Hash, despite the fact that having it in a HashMap<>
//      implies this already.
//      
//      It actually looks like the compiler is saying "You haven't specified that Iter<K> implies K
//      is Eq and Hash, but HashMap<K, V> requires this."
//      Rather than not understanding the transitive relationship it's mentioning the requirement
//      to me.
//      I guess this is to avoid confusion when something requires Eq and Hash because somewhere
//      down the line it puts the values in a HashMap.

//////    Iterator structures   //////
pub struct Iter<'a, K: 'a, V: 'a>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    order_iter: OrderIter<'a, K>,
    underlying_hash: &'a HashMap<Rc<K>, V>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
    where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        // Failing on `expect()` should never happen, all elements in the `order_link_map` should
        // be in `underlying_hash`, this is an invariant by design.
        // If something goes wrong here we *should* panic.
        self.order_iter.next()
            .map(|k| (k, self.underlying_hash.get(k)
                             .expect("OrderedDict corrupt! Ordered key missing in HashMap")))
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.order_iter.size_hint() }
}

pub struct IterMut<'a, K: 'a, V: 'a>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    order_iter: OrderIter<'a, K>,
    hidden: IterMutHidden<'a, K, V>,
}

struct IterMutHidden<'a, K: 'a, V: 'a>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    underlying_hash: &'a mut HashMap<Rc<K>, V>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        // Invariants from structure should mean the `unwraps()` are fine.
        //
        // NOTE I do something `unsafe` here, this is the justification:
        // 1)   I know the reference to be valid for the lifetime I give because it is the same
        //      lifetime as the mutable reference to the HashMap<> in my structure.
        // 2)   I know that no-one can get another reference to the same object at the same time
        //      because:
        //      - The `IterMut.hidden_struct` member is not public
        //      - I don't give out any other references to a given value
        //      - I have a mutable reference to the HashMap<>, which means no-one else can have any
        //        reference to any part of it.
        //
        // As an aside ... I'm surprised that the borrow checker is smart enough to figure out that
        // the `Iter.next()` method is OK.
        // I guess it figures that out because it can tell that the lifetime given as the returned
        // value from the method is the same lifetime as that in the structure.
        // Given that it knows there is already an immutably borrowed reference to the same
        // structure for the same amount of time, so it doesn't have to know anything else ... that
        // lifetime has already been validated.
        self.order_iter.next()
            .map(|k|
                 (k, unsafe {
                     &mut *{
                         self.hidden.underlying_hash.get_mut(k)
                             .expect("IterMut corrupt! Ordered key missing in HashMap")
                             as *mut V
                     }
                 }))
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.order_iter.size_hint() }
}

pub struct IntoIter<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    order_iter: OrderIntoIter<K>,
    underlying: HashMap<Rc<K>, V>,
}

// NOTE, the Debug trait is required so that expect() can be implemented.
impl<K, V> Iterator for IntoIter<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash + ::std::fmt::Debug {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        // After self.order_iter.next() is called, part of the specification of that function is
        // that it should remove all references to the Rc<K> returned inside its structure.
        // 
        // From the structure of OrderedDict, we know that this means there are only two strong
        // references to the Rc<K> left.
        // One is the `next_key` item that we're holding, the other is in self.underlying.
        let next_key = match self.order_iter.next() {
            Some(k) => k,
            None => { assert_eq!(self.underlying.len(), 0); return None },
        };

        // Here we remove the strong reference to Rc<k> in self.underlying, before unwrapping our
        // next_key reference to give the underlying value back to the caller.
        match self.underlying.remove(&next_key) {
            Some(v) => Some((Rc::try_unwrap(next_key)
                             .expect("Failed to unwrap key! There's an existing Rc<> still around"),
                             v)),
            None => panic!("IntoIter order_iter contained key not in hash map!"),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { self.order_iter.size_hint() }
}

pub struct Keys<'a, K: 'a, V: 'a>
    where K: ::std::cmp::Eq + ::std::hash::Hash {
        underlying: Iter<'a, K, V>
    }

impl<'a, K, V> Iterator for Keys<'a, K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.underlying.next().map(|x| x.0)
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.underlying.size_hint() }
}

pub struct Values<'a, K: 'a, V: 'a>
    where K: ::std::cmp::Eq + ::std::hash::Hash {
        underlying: Iter<'a, K, V>
    }

impl<'a, K, V> Iterator for Values<'a, K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        self.underlying.next().map(|x| x.1)
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.underlying.size_hint() }
}

pub struct ValuesMut<'a, K: 'a, V: 'a>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    inner: IterMut<'a, K, V>
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
            .map(|(_k, v)| v)
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}


//////     OrderedDict structure   //////
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    // map of keys to values
    underlying: HashMap<Rc<K>, V>,
    // Structure containing the order of keys inserted.
    order_link_map: OrderLinkMap<K>,
}

impl<K, V> OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    pub fn new() -> OrderedDict<K, V> { Default::default() }
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let ptr = Rc::new(k);
        match self.underlying.insert(Rc::clone(&ptr), v) {
            Some(v) => Some(v),
            None => {
                self.order_link_map.insert(Rc::clone(&ptr));
                None
            }
        }
    }
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where Rc<K>: ::std::borrow::Borrow<Q>,
          Q: ::std::cmp::Eq + ::std::hash::Hash {
        self.underlying.get(k)
    }
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            order_iter: self.order_link_map.iter(),
            underlying_hash: &self.underlying,
        }
    }
    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.order_link_map.remove(k);
        self.underlying.remove(k)
    }
    pub fn keys(&self) -> Keys<K, V> { Keys { underlying: self.iter() } }
    pub fn values(&self) -> Values<K, V> { Values { underlying: self.iter() } }
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            hidden: IterMutHidden {
                underlying_hash: &mut self.underlying,
            },
            order_iter: self.order_link_map.iter(),
        }
    }
    pub fn values_mut(&mut self) -> ValuesMut<K, V> { ValuesMut { inner: self.iter_mut() } }
    pub fn len(&self) -> usize { self.underlying.len() }
    pub fn is_empty(&self) -> bool { self.underlying.is_empty() }

    // TODO implement the below
    // pub fn entry(&mut self, key: K) -> Entry<K, V>
    // pub fn drain(&mut self) -> Drain<K, V>

    pub fn clear(&mut self) {
        self.underlying.clear();
        self.order_link_map.clear();
    }

    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where Rc<K>: ::std::borrow::Borrow<Q>,
          Q: ::std::cmp::Eq + ::std::hash::Hash {
        self.underlying.contains_key(k)
    }
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where Rc<K>: ::std::borrow::Borrow<Q>,
          Q: ::std::cmp::Eq + ::std::hash::Hash {
        self.underlying.get_mut(k)
    }

    pub fn retain<F>(&mut self, mut f: F)
    where F: FnMut(&K, &mut V) -> bool {
        let mut keys_to_remove: HashSet<Rc<K>> = HashSet::new();
        {
            let wrapped_closure = |k: &Rc<K>, v: &mut V| {
                if f(&*k, v) {
                    keys_to_remove.insert(Rc::clone(k));
                    true
                }
                else { false }
            };
            self.underlying.retain(wrapped_closure);
        }
        self.order_link_map.retain(|k| keys_to_remove.contains(k));
    }

    pub fn reserve(&mut self, additional: usize) {
        self.underlying.reserve(additional);
        self.order_link_map.reserve(additional);
    }
}

//////   Trait Implementations //////
// Mainly just taken from the HashMap implementation.

/*
 * NOTE:
 *      It's reasonably interesting that using derive(Default) works until you try and use
 *      Default::default().
 *      At that point rustc complains that Default isn't implemented for K or V, even though it's
 *      implemented for HashMap<K, V> and OrderLinkMap<K> whether implemented for K and V or not.
 */
impl<K, V> ::std::default::Default for OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    fn default() -> Self {
        OrderedDict {
            underlying: Default::default(),
            order_link_map: Default::default(),
        }
    }
}

impl<'a, Q, K, V> ::std::ops::Index<&'a Q> for OrderedDict<K, V>
where Rc<K>: ::std::borrow::Borrow<Q>,
      K: ::std::cmp::Eq + ::std::hash::Hash,
      Q: ::std::cmp::Eq + ::std::hash::Hash {
    type Output = V;
    fn index(&self, index: &Q) -> &V { self.underlying.index(index) }

}

impl<'a, K, V> IntoIterator for &'a OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, K, V> IntoIterator for &'a mut OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<K, V> IntoIterator for OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash + ::std::fmt::Debug {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            order_iter: self.order_link_map.into_iter(),
            underlying: self.underlying,
        }
    }
}

impl<K, V> FromIterator<(K, V)> for OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> OrderedDict<K, V> {
        let mut ret = OrderedDict::new();
        ret.extend(iter);
        ret
    }
}

impl<K, V> Extend<(K, V)> for OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            (iter.size_hint().0 + 1) / 2
        };
        self.reserve(reserve);
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, K, V> Extend<(&'a K, &'a V)> for OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash + ::std::marker::Copy,
      V: ::std::marker::Copy {
    fn extend<T: IntoIterator<Item = (&'a K,&'a V)>>(&mut self, iter: T) {
        self.extend(iter.into_iter().map(|(&key, &value)| (key, value)));
    }
}
