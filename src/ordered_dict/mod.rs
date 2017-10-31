use std::collections::HashMap;
use std::rc::Rc;
use std;

pub struct OrderedIter<'a> {
    inner: std::slice::Iter<'a, Rc<String>>,
}

// TODO Once all relevant methods have been implemented, change this to
// `impl Iterator for OrderedIter`
impl<'a> Iterator for OrderedIter<'a> {
    type Item = &'a String;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(std::ops::Deref::deref)
    }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

// These are the interfaces we want to implement.
pub trait OrderedDict {
    fn new() -> Self;
    fn insert(&mut self, k: String, v: usize) -> Option<usize>;
    fn get(&self, k: &String) -> Option<&usize>;
    fn iter(&self) -> OrderedIter;
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
    fn iter(&self) -> OrderedIter {
        OrderedIter { inner: self.order.iter() }
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
