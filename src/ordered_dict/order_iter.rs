use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapLink<K> {
    next: Option<Rc<K>>,
    prev: Option<Rc<K>>,
}

// NOTE I believe the only way to ensure that this is never instantiated with an incorrect `count`
// is to just not make it public.
// i.e. the only way to get the compiler to enforce I don't make mistakes is to create a new
// namespace outside the OrderLinkMap, which makes things more ugly than it's worth making them.
// (at least I think that way until I want to add something later on and I do it without ::new()
// and incorrectly).
pub struct OrderIter<'a, K: 'a> 
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

pub struct OrderIntoIter<K>
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
pub struct OrderLinkMap<K>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    head: Option<Rc<K>>,
    tail: Option<Rc<K>>,
    hash: HashMap<Rc<K>, MapLink<K>>,
}

impl<K> OrderLinkMap<K>
where K: ::std::cmp::Eq + ::std::hash::Hash {
    pub fn new() -> OrderLinkMap<K> {
        OrderLinkMap {
            head: None,
            tail: None,
            hash: HashMap::new(),
        }
    }
    // We check whether the entry already exists in the link map for correctness.
    // The fact that we don't insert anything if the entry already exists is never actually used.
    // It's simply something that makes sense for this data structure.
    pub fn insert(&mut self, k: Rc<K>) {
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

    pub fn clear(&mut self) {
        self.hash.clear();
        self.tail = None;
        self.head = None;
    }

    pub fn iter(&self) -> OrderIter<K> { OrderIter::new(self) }
    pub fn len(&self) -> usize { self.hash.len() }
    pub fn into_iter(self) -> OrderIntoIter<K> { OrderIntoIter { underlying: self } }
    // Don't need the same call-signature as HashMap<> ... we're not emulating anything here, just
    // creating something for our very limited use-case.
    pub fn get(&self, k: &Rc<K>) -> Option<&MapLink<K>> { self.hash.get(k) }

    // I don't like how un-performant this is..
    // Don't even bothen with a similar call signature to HashMap<>  ... this is a different thing.
    pub fn retain<F>(&mut self, mut f: F)
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

    pub fn reserve(&mut self, additional: usize) { self.hash.reserve(additional) }

    // Don't even bothen with a similar call signature to HashMap<> ... this is a different thing.
    pub fn remove(&mut self, k: &K) -> Option<MapLink<K>> {
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
