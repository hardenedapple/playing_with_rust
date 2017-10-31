use std::collections::{ HashMap, LinkedList };
use std::rc::Rc;
use std;

// Alternate thoughts
//
// Maybe I should use a vector for cache optimisation and to allow random access.
// The random access isn't really that useful at the moment, and it's entirely possible that the
// LinkedList implementation does something clever to try and optimise for caches.

// TODO
//  Things I don't really like:
//      I have to specify that K is Eq and Hash, despite the fact that having it in a HashMap<>
//      implies this already.
pub struct OrderedIter<'a, K: 'a, V: 'a> 
    where K: std::cmp::Eq + std::hash::Hash {
        // TODO Any way to make this neater?
        order_iter: std::collections::linked_list::Iter<'a, Rc<K>>,
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

// TODO
//      Allow deletion of elements in the hash map (i.e. use LinkedList<>)
//      Flesh out the interface by adding IntoIter implementations etc.
//      Parametrise the dict implementation over key and value types.

// --------------------------------------------------------------------------------
//  Safe implementation of an Ordered Set.
// --------------------------------------------------------------------------------

pub struct OrderedDict {
    underlying: HashMap<Rc<String>, usize>,
    order: LinkedList<Rc<String>>,
}

impl OrderedDict {
    pub fn new() -> OrderedDict {
        OrderedDict {
            underlying: HashMap::new(),
            order: LinkedList::new()
        }
    }
    pub fn insert(&mut self, k: String, v: usize) -> Option<usize> {
        let refcell = Rc::new(k);
        match self.underlying.insert(refcell.clone(), v) {
            Some(v) => Some(v),
            None => {
                self.order.push_back(refcell.clone());
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
    // pub fn remove(&mut self, k: &String) -> Option<usize> {
    //     // TODO Have to do something special here ...
    // }
    // fn keys(&self) -> OrderedKeys { OrderedKeys::new(self.order) }
    // fn values(&self) -> OrderedValues ..
    // fn iter_mut(&mut self)
    // fn keys_mut(&mut self)
    // fn values_mut(&mut self)
    // fn entry(&mut self, key: K)
    // fn len(&self) -> usize
    // fn is_empty(&self) -> bool
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
    }

    #[test]
    #[should_panic(expected = "Could not retrieve value from OrderedDict")]
    fn can_mark_missing() {
        let mut mydict = OrderedDict::new();
        mydict.insert(String::from("Other test"), 6);
        do_check(&mydict, String::from("Other tst"), 2, 6);
    }

    #[test]
    #[should_panic(expected = "\"2\"th element did not match")]
    fn can_mark_false() {
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
