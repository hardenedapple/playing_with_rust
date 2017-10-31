use std::collections:: HashMap;
use std::rc::Rc;
use std;

// Alternate thoughts
//
// Maybe I should use a vector for cache optimisation and to allow random access.
//  Using a Vec[] would mean that I don't have to store a pointer to a LinkedList element, but
//  could simply store an index into the vector.
//  It's essentially the same thing though, so I don't think it's *bette* so much as more
//  convenient.

// TODO
//  Things I don't really like:
//      I have to specify that K is Eq and Hash, despite the fact that having it in a HashMap<>
//      implies this already.
pub struct OrderedIter<'a, K: 'a, V: 'a> 
    where K: std::cmp::Eq + std::hash::Hash {
        // TODO Any way to make this neater?
        order_iter: std::slice::Iter<'a, Rc<K>>,
        underlying_hash: &'a HashMap<Rc<K>, V>,
}

impl<'a, K, V> Iterator for OrderedIter<'a, K, V>
    where K: std::cmp::Eq + std::hash::Hash {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        // TODO give a valid error message instead of just failing on `unwrap()`.
        // Failing on `unwrap()` should never happen, all elements in the `order_iter` should be in
        // `underlying_hash`, this is an invariant by design.
        // If something goes wrong here we *should* panic, but at the same time we should give
        // a nice error message to the user rather than a cryptic one that just comes from the
        // implementation details.
        self.order_iter.next()
            .map(std::ops::Deref::deref)
            .map(|k| { (k, self.underlying_hash.get(k).unwrap()) })
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.order_iter.size_hint() }
}

pub struct OrderedKeys<'a, K: 'a, V: 'a>
    where K: std::cmp::Eq + std::hash::Hash {
        underlying: OrderedIter<'a, K, V>
    }

impl<'a, K, V> Iterator for OrderedKeys<'a, K, V>
    where K: std::cmp::Eq + std::hash::Hash {
        type Item = &'a K;
        fn next(&mut self) -> Option<Self::Item> {
            self.underlying.next().map(|x| x.0)
        }
        fn size_hint(&self) -> (usize, Option<usize>) { self.underlying.size_hint() }
    }

pub struct OrderedValues<'a, K: 'a, V: 'a>
    where K: std::cmp::Eq + std::hash::Hash {
        underlying: OrderedIter<'a, K, V>
    }

impl<'a, K, V> Iterator for OrderedValues<'a, K, V>
    where K: std::cmp::Eq + std::hash::Hash {
        type Item = &'a V;
        fn next(&mut self) -> Option<Self::Item> {
            self.underlying.next().map(|x| x.1)
        }
        fn size_hint(&self) -> (usize, Option<usize>) { self.underlying.size_hint() }
    }


// TODO
//      Allow deletion of elements in the hash map (i.e. use LinkedList<>)
//      Flesh out the interface by adding IntoIter implementations etc.
//      Parametrise the dict implementation over key and value types.

// --------------------------------------------------------------------------------
//  Safe implementation of an Ordered Set.
// --------------------------------------------------------------------------------

pub struct OrderedDict {
    // map of keys to values
    underlying: HashMap<Rc<String>, usize>,
    // map of keys to positions in the vector
    map: HashMap<Rc<String>, usize>,
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
    order: Vec<Rc<String>>,
}

impl OrderedDict {
    pub fn new() -> OrderedDict {
        OrderedDict {
            underlying: HashMap::new(),
            map: HashMap::new(),
            order: Vec::new(),
        }
    }
    pub fn insert(&mut self, k: String, v: usize) -> Option<usize> {
        let refcell = Rc::new(k);
        match self.underlying.insert(refcell.clone(), v) {
            Some(v) => Some(v),
            None => {
                self.order.push(refcell.clone());
                self.map.insert(refcell.clone(), self.order.len() - 1);
                None
            }
        }
    }
    pub fn get(&self, k: &String) -> Option<&usize> { self.underlying.get(k) }
    pub fn iter(&self) -> OrderedIter<String, usize> {
        OrderedIter {
            order_iter: self.order.iter(),
            underlying_hash: &self.underlying
        }
    }
    // TODO This will change when I've implemented a LinkedList that I can rely on the internal
    // structure of.
    pub fn remove(&mut self, k: &String) -> Option<usize> {
        self.map.remove(k)
            .map(|v| Some(self.order.remove(v)));
        self.underlying.remove(k)
    }
    pub fn keys(&self) -> OrderedKeys<String, usize> {
        OrderedKeys {
            underlying: self.iter()
        }
    }
    pub fn values(&self) -> OrderedValues<String, usize> {
        OrderedValues { underlying: self.iter() }
    }
    // fn iter_mut(&mut self)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn do_check(set: &OrderedDict, key: String, index: usize, v: usize) {
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
        mydict.insert(String::from("Hello world"), 10);
        mydict.insert(String::from("Test string"), 15);
        mydict.insert(String::from("Other test"), 6);
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
        assert_eq!(items_in_order,
            vec![(String::from("Hello world"), 10),
                (String::from("Test string"), 15),
                (String::from("Other test"), 6),
            ]);
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
