extern crate rust_algorithms;

use rust_algorithms::ordered_dict::OrderedDict;

#[test]
fn create_and_check() {
    let mut mydict = rust_algorithms::ordered_dict::DictImpl::new();
    mydict.insert(String::from("Hello world"), 10);
    mydict.insert(String::from("Test string"), 15);
    mydict.insert(String::from("Other test"), 6);
    for item in mydict.iter() {
        println!("{:?}", item);
    }
}
