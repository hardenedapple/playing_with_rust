use std::collections:: HashMap;
use std::rc::Rc;
use std::iter::FromIterator;
use std::borrow::Borrow;

// Alternate thoughts
//
//  Using a Vec<T> to store the order of keys means that removal of keys is O(n).
//  This could be made O(1) by using a linked list.
//  The std::collections::LinkedList structure is opaque to the user, which means I can't store
//  pointers to Nodes in the middle of the list, and hence can't store which node needs removal.
//
//  I can see two main alternatives to get O(1) removal of keys.
//      1) Create my own Linked List structure, so I have access to the internal nodes.
//      2) Make a sort of linked list inside a hash table.
//          HashMap<Rc<K>, (Option<Rc<K>>, Option<Rc<K>>))
//          here the first entry in the tuple would be key for the "previous" entry, and the second
//          entry in the tuple would be the key for the "next" entry.
//      Either way, I will need to add in a "head" and "tail" member into the OrderedDict so as to
//      be able to start the iteration.
//  
//
// What about just keeping on using the Vec<T>?
// Unless removal of entries is a common occurance it shouldn't give too much of a performance
// impact.

pub struct Iter<'a, K: 'a, V: 'a> {
        order_iter: ::std::slice::Iter<'a, Rc<K>>,
        underlying_hash: &'a HashMap<Rc<K>, V>,
}

// TODO
//  Things I don't really like:
//      I have to specify that K is Eq and Hash, despite the fact that having it in a HashMap<>
//      implies this already.
impl<'a, K, V> Iterator for Iter<'a, K, V>
    where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        // TODO give a valid error message instead of just failing on `unwrap()`.
        // Failing on `unwrap()` should never happen, all elements in the `order_iter` should be in
        // `underlying_hash`, this is an invariant by design.
        // If something goes wrong here we *should* panic, but at the same time we should give
        // a nice error message to the user rather than a cryptic one that just comes from the
        // implementation details.
        self.order_iter.next()
            .map(::std::ops::Deref::deref)
            .map(|k| (k, self.underlying_hash.get(k).unwrap()))
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.order_iter.size_hint() }
}

pub struct OrderedKeys<'a, K: 'a, V: 'a>
    where K: ::std::cmp::Eq + ::std::hash::Hash {
        underlying: Iter<'a, K, V>
    }

impl<'a, K, V> Iterator for OrderedKeys<'a, K, V>
    where K: ::std::cmp::Eq + ::std::hash::Hash {
        type Item = &'a K;
        fn next(&mut self) -> Option<Self::Item> {
            self.underlying.next().map(|x| x.0)
        }
        fn size_hint(&self) -> (usize, Option<usize>) { self.underlying.size_hint() }
    }

pub struct OrderedValues<'a, K: 'a, V: 'a>
    where K: ::std::cmp::Eq + ::std::hash::Hash {
        underlying: Iter<'a, K, V>
    }

impl<'a, K, V> Iterator for OrderedValues<'a, K, V>
    where K: ::std::cmp::Eq + ::std::hash::Hash {
        type Item = &'a V;
        fn next(&mut self) -> Option<Self::Item> {
            self.underlying.next().map(|x| x.1)
        }
        fn size_hint(&self) -> (usize, Option<usize>) { self.underlying.size_hint() }
    }


// TODO
//      Allow deletion of elements in the hash map (i.e. use LinkedList<>)
//      Flesh out the interface by adding IntoIter implementations etc.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrderedDict<K, V> 
where K: ::std::cmp::Eq + ::std::hash::Hash {
    // map of keys to values
    underlying: HashMap<Rc<K>, V>,
    // map of keys to positions in the vector
    position_map: HashMap<Rc<K>, usize>,
    // TODO Implement my own doubly linked list.
    // That way I can rely on the implementation, and store Shared<T> pointers in the rest of the
    // OrderedDict structure.
    // When storing Shared<T> pointers I can make remove() O(1)
    //  This would be done in the same way as the reference python implementation of OrderedDict.
    //  That class stores 
    //      1) an underlying dictionary of keys and values.
    //      2) a map of keys to LinkedList nodes
    //      3) a linked list of elements
    // When removing a key, we use the map of keys to nodes in order to find the node to remove in
    // O(1) time.
    //
    // At the moment I use Vec<T>, which means the removal is O(n).
    // I can't use the provided LinkedList structure as the structure of the nodes is an
    // implementation detail, and there's no supported way to say "remove the element I have a
    // reference to".
    order: Vec<Rc<K>>,
}

impl<K, V> OrderedDict<K, V> 
where K: ::std::cmp::Eq + ::std::hash::Hash {
    pub fn new() -> OrderedDict<K, V> {
        OrderedDict {
            underlying: HashMap::new(),
            position_map: HashMap::new(),
            order: Vec::new(),
        }
    }
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let refcell = Rc::new(k);
        match self.underlying.insert(refcell.clone(), v) {
            Some(v) => Some(v),
            None => {
                self.order.push(refcell.clone());
                self.position_map.insert(refcell.clone(), self.order.len() - 1);
                None
            }
        }
    }
    // TODO Implement for Q where K: Borrow<Q>
    pub fn get(&self, k: &K) -> Option<&V> { self.underlying.get(k) }
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            order_iter: self.order.iter(),
            underlying_hash: &self.underlying
        }
    }
    // TODO This will change when I've implemented a LinkedList that I can rely on the internal
    // structure of.
    pub fn remove(&mut self, k: &K) -> Option<V> {
        self.position_map.remove(k)
            .map(|v| Some(self.order.remove(v)));
        self.underlying.remove(k)
    }
    pub fn keys(&self) -> OrderedKeys<K, V> {
        OrderedKeys {
            underlying: self.iter()
        }
    }
    pub fn values(&self) -> OrderedValues<K, V> {
        OrderedValues { underlying: self.iter() }
    }
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            hidden: IterMutHidden {
                underlying_hash: &mut self.underlying,
            },
            order_iter: self.order.iter(),
        }
    }
    // fn keys_mut(&mut self)
    // fn values_mut(&mut self)
    // fn entry(&mut self, key: K)
    pub fn len(&self) -> usize { self.order.len() }
    pub fn is_empty(&self) -> bool { self.order.is_empty() }
    // fn drain(&mut self) -> Drain<K, V>
    // fn clear(&mut self)
    // fn contains_key(&self, k: &Q)
    // get_mut
    // retain
    pub fn reserve(&mut self, additional: usize) {
        self.underlying.reserve(additional);
        self.position_map.reserve(additional);
        self.order.reserve(additional);
    }
}

// Implementations of traits just taken from the HashMap implementation.

impl<'a, K, V> ::std::ops::Index<&'a K> for OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    // TODO Implement for Q where K: Borrow<Q>
    type Output = V;
    fn index(&self, index: &K) -> &V {
        self.underlying.index(index)
    }

}

impl<'a, K, V> IntoIterator for &'a OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct IterMut<'a, K: 'a, V: 'a> {
    order_iter: ::std::slice::Iter<'a, Rc<K>>,
    hidden: IterMutHidden<'a, K, V>,
}

struct IterMutHidden<'a, K: 'a, V: 'a> {
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
            .map(::std::ops::Deref::deref)
            .map(|k|
                 (k, unsafe {
                     &mut *{self.hidden.underlying_hash.get_mut(k).unwrap() as *mut V}
                 }))
    }
}

impl<'a, K, V> IntoIterator for &'a mut OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct IntoIter<K, V> {
    inner: <Vec<Rc<K>> as IntoIterator>::IntoIter,
    underlying: HashMap<Rc<K>, V>,
}

impl<K, V> Iterator for IntoIter<K, V> 
where K: ::std::cmp::Eq + ::std::hash::Hash + ::std::fmt::Debug {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
            .map(|next_key| {
                // Invariants of the structure mean this should always work.
                // The Rc<K> is never given out (just references to the underlying K) so
                // Rc::try_unwrap() should work.
                //  (any references to the underlying K will have been dropped already, as their
                //  lifetimes would be limited by some reference to Rc<K>, and we have been given
                //  the Rc<> struct).
                // Everything added to the order vector is also added into the map, and they're
                // removed at the same time too, so the final unwrap() should work too.
                //
                // TODO Proper error messages in case my reasoning above is wrong?
                self.underlying.remove(next_key.borrow() as &K)
                    .map(|x| (Rc::try_unwrap(next_key).unwrap(), x))
                    .unwrap()
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

impl<K, V> IntoIterator for OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash + ::std::fmt::Debug {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.order.into_iter(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn do_check(set: &OrderedDict<String, usize>, key: String, index: usize, v: usize) {
        // Check fetching directly ...
        if let Some(value) = set.get(&key) {
            println!("{} should equal {}", value, v);
        } else {
            panic!("Could not retrieve value from OrderedDict");
        }

        // Check fetching in order ...
        match set.iter()
            .nth(index)
            .and_then(|first_string| { set.underlying.get(first_string.0) }) {
                Some(value) => assert_eq!(*value, v),
                None => panic!("\"{}\"th element did not match", index)
            }
    }

    #[test]
    fn create_and_check() {
        let mut mydict = OrderedDict::new();

        let inserted_items = vec![
            (String::from("Hello world"), 10),
            (String::from("Test string"), 15),
            (String::from("Other test"), 6)
        ];

        for item in inserted_items.iter() {
            mydict.insert(item.0.clone(), item.1);
        }

        do_check(&mydict, String::from("Hello world"), 0, 10);
        do_check(&mydict, String::from("Other test"), 2, 6);
        do_check(&mydict, String::from("Test string"), 1, 15);
        do_check(&mydict, String::from("Other test"), 2, 6);

        let keys_in_order = mydict.keys()
            .map(|x| x.clone())
            .collect::<Vec<_>>();
        assert_eq!(keys_in_order,
            vec![String::from("Hello world"),
                String::from("Test string"),
                String::from("Other test"),
            ]);

        let values_in_order = mydict.values()
            .map(|x| *x)
            .collect::<Vec<_>>();
        assert_eq!(values_in_order, vec![10, 15, 6]);

        let items_in_order = mydict.iter()
            .map(|(s, u)| (s.clone(), *u))
            .collect::<Vec<_>>();
        assert_eq!(items_in_order, inserted_items);

        // Testing IntoIterator for &'a
        for (map_item, check_item) in (&mydict).into_iter().zip(&inserted_items) {
            assert_eq!(map_item.0, &check_item.0);
            assert_eq!(map_item.1, &check_item.1);
        }
        // Testing IntoIterator for &'a mut
        for (map_item, check_item) in (&mut mydict).into_iter().zip(&inserted_items) {
            assert_eq!(map_item.0, &check_item.0);
            assert_eq!(map_item.1, &check_item.1);
        }

        // Testing IntoIterator for moved value
        for (map_item, check_item) in mydict.into_iter().zip(inserted_items) {
            assert_eq!(map_item, check_item);
        }
    }

    #[test]
    #[should_panic(expected = "Could not retrieve value from OrderedDict")]
    fn can_see_missing() {
        let mut mydict = OrderedDict::new();
        mydict.insert(String::from("Other test"), 6);
        do_check(&mydict, String::from("Other tst"), 2, 6);
    }

    #[test]
    #[should_panic(expected = "\"2\"th element did not match")]
    fn can_see_false() {
        let mut mydict = OrderedDict::new();
        mydict.insert(String::from("Other test"), 6);
        do_check(&mydict, String::from("Other test"), 2, 7);
    }


    /*
     * TODO
     *      Test ideas:
     *          All `.get()`, `.insert()`, `.remove()` operations give the same observable
     *          behaviour as the same ones applied to `HashMap<>`.
     */
}


// --------------------------------------------------------------------------------
//  Allow deletion of elements in the hash map.
// --------------------------------------------------------------------------------
// Switch Vec<> for LinkedList<>, add all the public HashMap<> methods, with special
// accounting for deletion.

// --------------------------------------------------------------------------------
//  Flesh out the interface to match the Rust HashMap.
// --------------------------------------------------------------------------------
// Implement IntoIter too ... iterate over the values in the `order` member, getting the full item
// from the `underlying` member.


// --------------------------------------------------------------------------------
//  Generalise over arbitrary keys and values.
// --------------------------------------------------------------------------------
