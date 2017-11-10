use std::collections::{ HashMap, HashSet };
use std::rc::Rc;
use std::iter::FromIterator;

// TODO
//      Restructure the file
//          Put some things in modules,
//          Add comment block markers to distinguish parts of the code that belong together.
//          Put some modules into other files.
//      More tests
//          Test for the size_hint() of all the different iterators.
//          Add some randomised tests to check the behaviour against that of HashMap<>.
//          Include tests of the ordering in the randomised tests.
//          Randomised tests, checking that the ordering and underlying hash match.
//      Finish Implementation
//          I still have to implement drain(&mut self) and entry(&mut self).

pub struct Iter<'a, K: 'a, V: 'a>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    order_iter: OrderIter<'a, K>,
    underlying_hash: &'a HashMap<Rc<K>, V>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct MapLink<K> {
    next: Option<Rc<K>>,
    prev: Option<Rc<K>>,
}

// NOTE I believe the only way to ensure that this is never instantiated with an incorrect `count`
// is to just not make it public.
// i.e. there is no way to compiler enforce against my errors in that instance.
struct OrderIter<'a, K: 'a> 
where K: ::std::cmp::Eq + ::std::hash::Hash {
    current: Option<&'a Rc<K>>,
    order_link_map: &'a OrderLinkMap<K>,
    count: usize,
}

impl<'a, K> OrderIter<'a, K>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    fn new(link_map: &'a OrderLinkMap<K>) -> Self {
        OrderIter {
            current: None,
            order_link_map: link_map,
            count: link_map.len(),
        }
    }
}

impl<'a, K> Iterator for OrderIter<'a, K>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.current = match self.current {
            Some(k) => match self.order_link_map.get(k) {
                Some(map_link) => map_link.next.iter().next(),
                None => panic!("Iter over ordered links \"current\" points to missing key!"),
            },
            None => self.order_link_map.head.iter().next()
        };
        match self.current {
            Some(k) => { self.count -= 1; Some(&**k) },
            None => { assert_eq!(self.count, 0); None },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) { (self.count, Some(self.count)) }
}

struct OrderIntoIter<K>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    underlying: OrderLinkMap<K>
}

/*
 * n.b. This returns Rc<K> instead of K because its entire purpose is to work with the OrderedDict
 * IntoIter structure.
 *
 * When this structure is calling .next(), it still has a strong reference to the same Rc<K> in the
 * underlying hash of OrderedDict.
 *
 * This means that there is no way for us to return K.
 *
 * That structure can't use the OrderIter above because there would be no way for it to obtain a K
 * value (without cloning) from a reference to a K.
 */
impl <K> Iterator for OrderIntoIter<K>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = Rc<K>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_key = match self.underlying.head {
            Some(ref k) =>  k.clone(),
            // Short circuit here, maybe should add some assert!() statements.
            // assert_eq!(self.underlying.len(), 0);
            None => return None,
        };

        // This should remove both the Rc<> strong count in the underlying.hash and the one in
        // the underlying.head.
        // There should only be a strong reference left in the OrderedDict.underlying and in
        // next_key.
        self.underlying.head = self.underlying.remove(&next_key)
            .expect("OrderedLinkMap head was not in OrderedLinkMap hash")
            .next;

        Some(next_key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) { 
        (self.underlying.len(), Some(self.underlying.len()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OrderLinkMap<K>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    head: Option<Rc<K>>,
    tail: Option<Rc<K>>,
    hash: HashMap<Rc<K>, MapLink<K>>,
}

impl<K> OrderLinkMap<K>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    fn new() -> OrderLinkMap<K> {
        OrderLinkMap {
            head: None,
            tail: None,
            hash: HashMap::new(),
        }
    }
    // We check whether the entry already exists in the link map for correctness.
    // The fact that we don't insert anything if the entry already exists is never actually used.
    // It's simply something that makes sense for this data structure.
    fn insert(&mut self, k: Rc<K>) {
        if self.hash.contains_key(&k) {
            return;
        }
        match self.tail {
            Some(ref x) => {
                self.hash.get_mut(x)
                    .expect("OrderLinkMap corrupt! self.tail points to a missing key.")
                    .next = Some(k.clone());

                self.hash.insert(k.clone(), MapLink {
                    next: None,
                    prev: Some(x.clone()),
                });

                assert!(self.head.is_some());
            },
            None => {
                assert!(self.head.is_none());
                self.head = Some(k.clone());
                self.hash.insert(k.clone(), MapLink {
                    next: None,
                    prev: None
                });
            }
        }
        self.tail = Some(k);
    }

    fn clear(&mut self) {
        self.hash.clear();
        self.tail = None;
        self.head = None;
    }

    fn iter(&self) -> OrderIter<K> { OrderIter::new(self) }
    fn len(&self) -> usize { self.hash.len() }
    fn into_iter(self) -> OrderIntoIter<K> { OrderIntoIter { underlying: self } }
    // Don't need the same call-signature as HashMap<> ... we're not emulating anything here, just
    // creating something for our very limited use-case.
    fn get(&self, k: &Rc<K>) -> Option<&MapLink<K>> { self.hash.get(k) }

    // I don't like how un-performant this is..
    // Don't even bothen with a similar call signature to HashMap<>  ... this is a different thing.
    fn retain<F>(&mut self, mut f: F)
    where F: FnMut(&K) -> bool {
        let keys_to_remove = self.hash.iter_mut()
            .filter_map(
                |(k, _value)|
                if f(k) {
                    None
                } else {
                    Some(k.clone())
                })
            .collect::<Vec<_>>();
        for key in keys_to_remove.iter() {
            self.remove(&*key);
        }
    }

    fn reserve(&mut self, additional: usize) { self.hash.reserve(additional) }

    // Don't even bothen with a similar call signature to HashMap<> ... this is a different thing.
    fn remove(&mut self, k: &K) -> Option<MapLink<K>> {
        if let Some(x) = self.hash.remove(k)  {
            match x.next {
                // x.next == None   <=>  k is the tail
                Some(ref next_link) => {
                    self.hash.get_mut(next_link)
                        .expect("E:Remove 225")
                        .prev = x.prev.clone();
                },
                None => {
                    self.tail = x.prev.clone()
                }
            };
            match x.prev {
                // x.prev == None   <=>  k is the head
                Some(ref prev_link) => {
                    self.hash.get_mut(prev_link)
                        .expect("E:Remove 2")
                        .next = x.next.clone()
                },
                None => {
                    self.head = x.next.clone()
                }
            };
            Some(x)
        } else { None }
    }
}

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
    pub fn new() -> OrderedDict<K, V> {
        OrderedDict {
            underlying: HashMap::new(),
            order_link_map: OrderLinkMap::new(),
        }
    }
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let ptr = Rc::new(k);
        match self.underlying.insert(ptr.clone(), v) {
            Some(v) => Some(v),
            None => {
                self.order_link_map.insert(ptr.clone());
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
            order_iter: self.order_link_map.iter(),
        }
    }
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut { inner: self.iter_mut() }
    }
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
                if f(&*k, v) { false }
                else {
                    keys_to_remove.insert(k.clone());
                    true
                }
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

// Implementations of traits just taken from the HashMap implementation.

impl<'a, Q, K, V> ::std::ops::Index<&'a Q> for OrderedDict<K, V>
where Rc<K>: ::std::borrow::Borrow<Q>,
      K: ::std::cmp::Eq + ::std::hash::Hash,
      Q: ::std::cmp::Eq + ::std::hash::Hash {
    type Output = V;
    fn index(&self, index: &Q) -> &V {
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

impl<'a, K, V> IntoIterator for &'a mut OrderedDict<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct IntoIter<K, V>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    order_iter: OrderIntoIter<K>,
    underlying: HashMap<Rc<K>, V>,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn do_check(set: &OrderedDict<String, usize>, key: String, index: usize, v: usize) {
        // Check fetching directly ...
        assert_eq!(
            set.get(&key).expect("Could not retrieve value from OrderedDict"),
            &v
        );

        // Check fetching in order ...
        match set.iter()
            .nth(index)
            .and_then(|first_string| { set.underlying.get(first_string.0) }) {
                Some(value) => assert_eq!(*value, v),
                None => panic!("\"{}\"th element did not match", index)
            }
    }

    fn create_default() ->
        (OrderedDict<String, usize>, Vec<(String, usize)>) {
        let mut mydict = OrderedDict::new();

        let inserted_items = vec![
            (String::from("Hello world"), 10),
            (String::from("Test string"), 15),
            (String::from("Other test"), 6)
        ];

        for item in inserted_items.iter() {
            mydict.insert(item.0.clone(), item.1);
        }

        (mydict, inserted_items)
    }

    fn assert_keys(orig_dict: &OrderedDict<String, usize>, expected: Vec<String>) {
        let keys_in_order = orig_dict.keys()
            .map(|x| x.clone())
            .collect::<Vec<_>>();
        assert_eq!(keys_in_order, expected);
    }

    #[test]
    fn iter_values() {
        let (mydict, inserted_items) = create_default();

        let values_in_order = mydict.values()
            .map(|x| *x)
            .collect::<Vec<_>>();
        assert_eq!(values_in_order,
                   inserted_items.into_iter().map(|x| x.1)
                    .collect::<Vec<_>>());
    }

    #[test]
    fn simple_insert() {
        let (mydict, _inserted_items) = create_default();

        do_check(&mydict, String::from("Hello world"), 0, 10);
        do_check(&mydict, String::from("Other test"), 2, 6);
        do_check(&mydict, String::from("Test string"), 1, 15);
        do_check(&mydict, String::from("Other test"), 2, 6);

        assert_keys(&mydict,
            vec![String::from("Hello world"),
                String::from("Test string"),
                String::from("Other test"),
            ]);
    }

    #[test]
    fn simple_remove() {
        let (mut mydict, _inserted_items) = create_default();
        let remove_key = String::from("Hello world");
        mydict.remove(&remove_key);
        let keys_in_order = mydict.keys()
            .map(|x| x.clone())
            .collect::<Vec<_>>();
        assert_eq!(keys_in_order,
                   vec![String::from("Test string"),
                   String::from("Other test")]);
    }

    #[test]
    fn retain() {
        // Original test.
        // I'm keeping it here because I want to check that the way retain works is the same
        // between my implementation and the standard.
        // i.e. TODO Once I have everything working ... Ensure that I've gotten the boolean the
        // right way around.
        //
        //
        // mydict.insert(String::from("test"), 8);
        // // n.b. I don't really follow this ...
        // // The docs say "remove all pairs (k, v) such that f(&k, &mut v) returns false"
        // // This should mean that the below function removes all (k, v) such that x != 't', which is
        // // the opposite of what happens.
        // mydict.retain(
        //     |k, _v| {
        //     match k.chars().next() {
        //         Some(x) => x == 't',
        //         None => false
        //     }
        // });
        let (mut mydict, _inserted_items) = create_default();
        mydict.retain(
            |k, _v| {
                match k.chars().next() {
                    Some(x) => x == 'O',
                    None => false
                }
            });
        assert_keys(&mydict,
                    vec![String::from("Hello world"),
                        String::from("Test string")]);
    }

    #[test]
    fn iter_ref() {
        let (mydict, inserted_items) = create_default();

        let items_in_order = mydict.iter()
            .map(|(s, u)| (s.clone(), *u))
            .collect::<Vec<_>>();
        assert_eq!(items_in_order, inserted_items);
    }

    #[test]
    fn into_iter() {
        let (mydict, inserted_items) = create_default();

        // Testing IntoIterator for moved value
        for (map_item, check_item) in mydict.into_iter().zip(inserted_items) {
            assert_eq!(map_item, check_item);
        }
    }

    #[test]
    fn iter_mut() {
        let (mut mydict, inserted_items) = create_default();

        // Testing IntoIterator for &'a mut
        for (map_item, check_item) in (&mut mydict).into_iter().zip(&inserted_items) {
            assert_eq!(map_item.0, &check_item.0);
            assert_eq!(map_item.1, &check_item.1);
        }
    }

    #[test]
    fn into_iter_ref() {
        let (mydict, inserted_items) = create_default();

        // Testing IntoIterator for &'a
        for (map_item, check_item) in (&mydict).into_iter().zip(&inserted_items) {
            assert_eq!(map_item.0, &check_item.0);
            assert_eq!(map_item.1, &check_item.1);
        }
    }

    #[test]
    fn iterate_mutable_values() {
        // Check that I can change the values in the hash map using the references I get from
        // values_mut()
        let (mut mydict, _inserted_items) = create_default();
        let mut new_value = 10;
        for value in mydict.values_mut() {
            *value = new_value;
            new_value += 1;
        }
        new_value -= mydict.len();
        // Iterate over values() for the check so that values_mut() isn't checking it's own work.
        for value in mydict.values() {
            assert_eq!(*value, new_value);
            new_value += 1;
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
    #[should_panic(expected = "assertion failed: `(left == right)`")]
    fn can_see_false() {
        let mut mydict = OrderedDict::new();
        mydict.insert(String::from("Other test"), 6);
        do_check(&mydict, String::from("Other test"), 2, 7);
    }

    #[test]
    fn iterate_order_link_map() {
        let (mydict, inserted_items) = create_default();
        // TODO Maybe make this a little less ugly.
        assert_eq!(mydict.order_link_map.iter().map(|x| x.clone()).collect::<Vec<String>>(),
                inserted_items.iter().map(|x| x.0.clone()).collect::<Vec<String>>());
    }


    /*
     * TODO
     *      Test ideas:
     *          --  All `.get()`, `.insert()`, `.remove()` operations give the same observable
     *              behaviour as the same ones applied to `HashMap<>`.
     *          --  After all method calls, the keys in self.underlying_hash and
     *              self.order_link_map are the same.
     */
}
