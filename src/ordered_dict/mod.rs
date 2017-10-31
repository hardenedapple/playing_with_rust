use std::collections::HashMap;
use std::rc::Rc;
use std;

// TODO
//  Things I don't really like:
//      I have to specify that K is Eq and Hash, despite the fact that having it in a HashMap<>
//      implies this already.
pub struct OrderedIter<'a, K: 'a, V: 'a> 
    where K: std::cmp::Eq + std::hash::Hash {
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

// These are the interfaces we want to implement.
pub trait OrderedDict {
    fn new() -> Self;
    fn insert(&mut self, k: String, v: usize) -> Option<usize>;
    fn get(&self, k: &String) -> Option<&usize>;
    fn iter(&self) -> OrderedIter<String, usize>;
    // fn keys(&self) -> OrderedKeys;
    // fn values(&self) -> OrderedValues;
    // fn remove(&mut self, k: &String) -> Option<(String, usize)>;
}

// TODO
//      Allow deletion of elements in the hash map (i.e. use LinkedList<>)
//      Flesh out the interface by adding IntoIter implementations etc.
//      Parametrise the dict implementation over key and value types.

// --------------------------------------------------------------------------------
//  Safe implementation of an Ordered Set.
// --------------------------------------------------------------------------------

pub struct DictImpl {
    underlying: HashMap<Rc<String>, usize>,
    order: Vec<Rc<String>>,
}

impl OrderedDict for DictImpl {
    fn new() -> DictImpl {
        DictImpl {
            underlying: HashMap::new(),
            order: Vec::new()
        }
    }
    fn insert(&mut self, k: String, v: usize) -> Option<usize> {
        let refcell = Rc::new(k);
        self.order.push(refcell.clone());
        self.underlying.insert(refcell.clone(), v)
    }
    fn get(&self, k: &String) -> Option<&usize> { self.underlying.get(k) }
    fn iter(&self) -> OrderedIter<String, usize> {
        OrderedIter {
            order_iter: self.order.iter(),
            underlying_hash: &self.underlying
        }
    }
    // fn keys(&self) -> OrderedKeys { OrderedKeys::new(self.order) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn do_check(set: DictImpl, key: String, index: usize, v: usize) {
        // Check fetching directly ...
        if let Some(value) = set.get(&key) {
            println!("{} should equal {}", value, v);
        } else {
            panic!("Could not retrieve value from DictImpl");
        }

        // Check fetching in order ...
        let original_str = &set.order[index];
        if let Some(value) = set.underlying.get(original_str) {
            println!("{} should equal {}", value, v);
        } else {
            println!("Something is going wrong!!!");
        }
    }

    #[test]
    fn create_and_check() {
        let mut mydict = DictImpl::new();
        mydict.insert(String::from("Hello world"), 10);
        mydict.insert(String::from("Test string"), 15);
        mydict.insert(String::from("Other test"), 6);
        do_check(mydict, String::from("Hello world"), 0, 10);

        // let mut iterator = iterator;
        // assert_eq!(iterator.next(), ...);
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
