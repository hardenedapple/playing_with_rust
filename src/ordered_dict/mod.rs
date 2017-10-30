use std::collections::HashMap;
use std::rc::Rc;
use std;

// TODO
//      Parametrise the test_*() functions over the three different types.
//      Put each implementation into a different module.
//      Allow deletion of elements in the hash map (i.e. use LinkedList<>)
//      Flesh out the interface by adding IntoIter implementations etc.
//      Parametrise the dict implementation over key and value types.

// This implementation has a problem ...
// I have to rely on the String in the HashMap not being moved anywhere, and I'm
// not in control of that HashMap.
//
// For now just hard-code the type ...
// That's not where the interesting things are

pub mod broken_attempt {
    use super::*;
    pub struct OrderedDict {
        underlying: HashMap<String, usize>,
        order: Vec<* const String>,
    }

    impl OrderedDict {
        pub fn new() -> OrderedDict {
            OrderedDict {
                underlying: HashMap::new(),
                order: Vec::new()
            }
        }
        pub fn insert(&mut self, k: String, v: usize) -> Option<usize> {
            let ptr: * const String = &k;
            self.order.push(ptr);
            self.underlying.insert(k, v)
        }
        pub fn get(&self, k: &String) -> Option<&usize> {
            self.underlying.get(k)
        }
    }

    // This is not "safe" (in the ordinary terminology, not the Rust one).
    // The HashMap<> is perfectly within its rights to move the underlying String it owns.
    // In fact, sometimes you even get a segmentation fault.
    pub fn do_check(dict: OrderedDict, key: String, index: usize, v: usize) {
        if let Some(value) = dict.get(&key) {
            println!("{} should equal {}", value, v);
        } else {
            panic!("Could not retrieve value from OrderedDict2");
        }

        let ptr = dict.order[index];
        let reference: &String = unsafe { &*ptr };
        if let Some(value) = dict.underlying.get(reference) {
            println!("{} should equal {}", value, v);
        } else {
            println!("Something is going wrong!!!");
        }
    }


    pub fn test() {
        let mut mydict = OrderedDict::new();
        mydict.insert(String::from("Hello world"), 10);
        do_check(mydict, String::from("Hello world"), 0, 10);
    }
}

// --------------------------------------------------------------------------------
//  Better implementation
// --------------------------------------------------------------------------------

// We can be much more confident that the HashMap doesn't cause the underlying String
// representation to move its &str slice.
// I'd be pretty happy using this for a medium sized script that no-one else relied on.
pub mod incorrect_yet_working {
    use super::*;
    pub struct OrderedDict {
        underlying: HashMap<String, usize>,
        order: Vec<(* const u8, usize)>,
    }


    impl OrderedDict {
        pub fn new() -> OrderedDict {
            OrderedDict {
                underlying: HashMap::new(),
                order: Vec::new()
            }
        }
        pub fn insert(&mut self, k: String, v: usize) -> Option<usize> {
            let ptr = (&k as &str).as_ptr();
            let len = k.len();
            self.order.push((ptr, len));
            self.underlying.insert(k, v)
        }
        pub fn get(&self, k: &String) -> Option<&usize> {
            self.underlying.get(k)
        }
    }

    pub fn do_check_better(dict: OrderedDict, key: String, index: usize, v: usize) {
        // Check fetching directly ...
        if let Some(value) = dict.get(&key) {
            println!("{} should equal {}", value, v);
        } else {
            panic!("Could not retrieve value from OrderedDict2");
        }

        // Check fetching in order ...
        let original_str = { 
            let (ptr,len) = dict.order[index];
            let slice: &[u8] = unsafe { std::slice::from_raw_parts(ptr, len) };
            // n.b. we know it's a valid UTF8 -- we stored it from a String
            std::str::from_utf8(slice).unwrap()
        };
        if let Some(value) = dict.underlying.get(original_str) {
            println!("{} should equal {}", value, v);
        } else {
            println!("Something is going wrong!!!");
        }
    }


    pub fn test() {
        let mut mydict = OrderedDict::new();
        mydict.insert(String::from("Hello world"), 10);
        do_check_better(mydict, String::from("Hello world"), 0, 10);
    }
}

// --------------------------------------------------------------------------------
//  Safe implementation of an Ordered Set.
// --------------------------------------------------------------------------------

pub mod correct_unfinished {
    use super::*;
    pub struct OrderedDict {
        underlying: HashMap<Rc<String>, usize>,
        order: Vec<Rc<String>>,
    }

    impl OrderedDict {
        pub fn new() -> OrderedDict {
            OrderedDict {
                underlying: HashMap::new(),
                order: Vec::new()
            }
        }
        pub fn insert(&mut self, k: String, v: usize) -> Option<usize> {
            let refcell = Rc::new(k);
            self.order.push(refcell.clone());
            self.underlying.insert(refcell.clone(), v)
        }
        pub fn get(&self, k: &String) -> Option<&usize> {
            self.underlying.get(k)
        }
    }

    pub fn do_check_alt(dict: OrderedDict, key: String, index: usize, v: usize) {
        // Check fetching directly ...
        if let Some(value) = dict.get(&key) {
            println!("{} should equal {}", value, v);
        } else {
            panic!("Could not retrieve value from OrderedDict");
        }

        // Check fetching in order ...
        let original_str = &dict.order[index];
        if let Some(value) = dict.underlying.get(original_str) {
            println!("{} should equal {}", value, v);
        } else {
            println!("Something is going wrong!!!");
        }
    }


    pub fn test() {
        let mut mydict = OrderedDict::new();
        mydict.insert(String::from("Hello world"), 10);
        do_check_alt(mydict, String::from("Hello world"), 0, 10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        println!("Finished version");
        correct_unfinished::test();
        println!("Second attempt");
        incorrect_yet_working::test();
    }

    #[test]
    #[ignore]
    fn problematic_test() {
        // Original attempt .. will probably segfault
        // Hence why its disabled by default.
        broken_attempt::test();
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
