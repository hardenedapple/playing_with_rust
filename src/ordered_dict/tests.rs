use super::*;

// TODO
//      Could I implement assert_iterator_matches() by taking an immutable reference, but defining
//      an intermediate type that is basically a tuple, but where (&String, &usize) implements
//      PartialEq<(&String, &mut usize> and other similar implementations are provided.

fn do_fetch_check(set: &OrderedDict<String, usize>, key: String, index: usize, v: usize) {
    assert_eq!(
        set.get(&key).expect("Could not retrieve value from OrderedDict"),
        &v
    );

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

    for item in inserted_items.iter() { mydict.insert(item.0.clone(), item.1); }

    (mydict, inserted_items)
}

// NOTE:
//  We can't define this function to be generic over mutable/immutable references.
//  i.e. we can't define it so that expected can be either mutable or immutable.
//  Because we want to use this when testing mutable references, we've chosen the more restrictive
//  type and made all other uses of the function pass us a mutable reference.
//
//  This is a bit of a pain.
fn assert_iterator_matches<'a, V, T, F, R>(mut iterator: V,
                           expected: &'a mut Vec<(String, usize)>,
                           conversion: F)
where V: ::std::iter::Iterator<Item = T>,
      T: ::std::cmp::PartialEq<R> + ::std::fmt::Debug,
      F: Fn(&'a mut (String, usize)) -> R,
      R: ::std::fmt::Debug {
    let mut original_length = expected.len();

    assert_eq!(iterator.size_hint(),
                (original_length, Some(original_length)));

    for should in expected.into_iter() {
        let gotten = iterator.next()
            .expect("ERROR: There are more elements in expected than iterator!");
        assert_eq!(gotten, conversion(should));
        original_length -= 1;
        assert_eq!(iterator.size_hint(),
                    (original_length, Some(original_length)));
    }
}

fn get_key(x: &mut(String, usize)) -> &String { &x.0 }
fn get_value(x: &mut(String, usize)) -> &usize { &x.1 }
fn get_item(x: &mut(String, usize)) -> (&String, &usize) { (&x.0, &x.1) }
fn clone_item(x: &mut(String, usize)) -> (String, usize) { (x.0.clone(), x.1) }
fn get_mut_value(x: &mut(String, usize)) -> &mut usize { &mut x.1 }
fn get_mut_item(x: &mut(String, usize)) -> (&String, &mut usize) { (&x.0, &mut x.1) }

#[test]
fn iter_values() {
    let (mydict, mut inserted_items) = create_default();
    assert_iterator_matches(mydict.values(), &mut inserted_items, get_value);
}

#[test]
fn simple_insert() {
    let (mydict, mut inserted_items) = create_default();

    do_fetch_check(&mydict, String::from("Hello world"), 0, 10);
    do_fetch_check(&mydict, String::from("Other test"), 2, 6);
    do_fetch_check(&mydict, String::from("Test string"), 1, 15);
    do_fetch_check(&mydict, String::from("Other test"), 2, 6);

    assert_iterator_matches(mydict.keys(), &mut inserted_items, get_key);
}

#[test]
fn simple_remove() {
    let (mut mydict, mut inserted_items) = create_default();
    let remove_key = String::from("Hello world");
    mydict.remove(&remove_key);
    inserted_items.retain(|&(ref k, _v)| { k != &remove_key });
    assert_iterator_matches({ &mydict }.into_iter(), &mut inserted_items, get_item);
}

#[test]
fn retain() {
    let (mut mydict, mut inserted_items) = create_default();
    fn check_key(k: &String) -> bool {
        match k.chars().next() {
            Some(x) => x == 'O',
            None => false
        }
    }

    mydict.retain(|k, _v| check_key(k));
    inserted_items.retain(|&(ref k, _v)| check_key(k));

    assert_iterator_matches(mydict.keys(), &mut inserted_items, get_key);
}

#[test]
fn iter_ref() {
    let (mydict, mut inserted_items) = create_default();
    assert_iterator_matches(mydict.iter(), &mut inserted_items, get_item);
}

#[test]
fn into_iter() {
    let (mydict, mut inserted_items) = create_default();
    assert_iterator_matches(mydict.into_iter(), &mut inserted_items, clone_item);
    // Something of interest ... when trying to re-use the get_item() function we get different
    // errors depending on the order that the symmetric trait PartialEq<> is specified.
    //
    // assert_iterator_matches(mydict.into_iter(), &inserted_items, get_item);
    //
    // If the trait bounds on assert_iterator_matches() are
    //      R: ::std::cmp::PartialEq<T> + ::std::fmt::Debug,
    //      T: ::std::fmt::Debug
    // and the assert_eq!() comparison is switched around.
    // Then we get the error message below.
    //
    // src/ordered_dict/tests.rs|140 col 5 error 277| the trait bound `(&std::string::String, &usize): std::cmp::PartialEq<(std::string::String, usize)>` is not satisfied
    // ||     |
    // || 140 |     assert_iterator_matches(mydict.into_iter(), &inserted_items, get_item);
    // ||     |     ^^^^^^^^^^^^^^^^^^^^^^^ can't compare `(&std::string::String, &usize)` with `(std::string::String, usize)`
    // ||     |
    // ||     = help: the trait `std::cmp::PartialEq<(std::string::String, usize)>` is not implemented for `(&std::string::String, &usize)`
    // ||     = note: required by `ordered_dict::tests::assert_iterator_matches`
    // src/ordered_dict/tests.rs|156 col 5 error 271| type mismatch resolving `<fn(&(std::string::String, usize)) -> (&std::string::String, &usize) {ordered_dict::tests::get_item} as std::ops::FnOnce<(&(std::string::String, usize),)>>::Output == (std::string::String, usize)`
    //
    // But with the trait bounds as they are, we get the error
    //
    // ||     |
    // || 156 |     assert_iterator_matches(mydict.into_iter(), &inserted_items, get_item);
    // ||     |     ^^^^^^^^^^^^^^^^^^^^^^^ expected reference, found struct `std::string::String`
    // ||     |
    // ||     = note: expected type `(&std::string::String, &usize)`
    // ||                found type `(std::string::String, usize)`
    // ||     = note: required by `ordered_dict::tests::assert_iterator_matches`
    //
    // I'm guessing this is because there is no implementation of std::cmp::PartialEq<T> for
    // (&String, &usize), and hence rustc doesn't even get around to checking if the type given
    // fits the trait bounds.
    // Instead I guess it justs errors out because the types don't match.
}

#[test]
fn iter_mut() {
    let (mut mydict, mut inserted_items) = create_default();
    // This use is the main place that requires assert_iterator_matches() to take a mutable
    // reference.
    // Without it allowing this we get the error below, which I believe simply stems from there not
    // being a PartialEq<> implementation for tuples of references.
    //
    // src/ordered_dict/tests.rs|183 col 5 error 271| type mismatch resolving `<fn(&(std::string::String, usize)) -> (&std::string::String, &usize) {ordered_dict::tests::get_item} as std::ops::FnOnce<(&(std::string::String, usize),)>>::Output == (&std::string::String, &mut usize)`
    // ||     |
    // || 183 |     assert_iterator_matches({ &mut mydict }.into_iter(), &inserted_items, get_item);
    // ||     |     ^^^^^^^^^^^^^^^^^^^^^^^ types differ in mutability
    // ||     |
    // ||     = note: expected type `(&std::string::String, &usize)`
    // ||                found type `(&std::string::String, &mut usize)`
    // ||     = note: required by `ordered_dict::tests::assert_iterator_matches`
    assert_iterator_matches({ &mut mydict }.into_iter(), &mut inserted_items, get_mut_item);

    // Just check that we can change the items.
    let mut new_value = 10;
    for ((_, value), should_store)
        in {&mut mydict}.into_iter().zip({&mut inserted_items}.into_iter()) {
        *value = new_value;
        should_store.1 = new_value;
        new_value += 1;
    }
    assert_iterator_matches({ &mut mydict }.into_iter(), &mut inserted_items, get_mut_item);
}

#[test]
fn into_iter_ref() {
    let (mydict, mut inserted_items) = create_default();
    assert_iterator_matches({ &mydict }.into_iter(), &mut inserted_items, get_item);
}

#[test]
fn iterate_mutable_values() {
    // Check that I can change the values in the hash map using the references I get from
    // values_mut()
    let (mut mydict, mut inserted_items) = create_default();

    assert_iterator_matches(mydict.values_mut(), &mut inserted_items, get_mut_value);

    // Just check that we can change the items.
    let mut new_value = 10;
    for (value, should_store) in mydict.values_mut().zip({&mut inserted_items}.into_iter()) {
        *value = new_value;
        should_store.1 = new_value;
        new_value += 1;
    }
    assert_iterator_matches(mydict.values_mut(), &mut inserted_items, get_mut_value);
}

#[test]
#[should_panic(expected = "Could not retrieve value from OrderedDict")]
fn can_see_missing() {
    let mut mydict = OrderedDict::new();
    mydict.insert(String::from("Other test"), 6);
    do_fetch_check(&mydict, String::from("Other tst"), 2, 6);
}

#[test]
#[should_panic(expected = "assertion failed: `(left == right)`")]
fn can_see_false() {
    let mut mydict = OrderedDict::new();
    mydict.insert(String::from("Other test"), 6);
    do_fetch_check(&mydict, String::from("Other test"), 2, 7);
}

#[test]
fn iterate_order_link_map() {
    let (mydict, mut inserted_items) = create_default();
    assert_iterator_matches(mydict.order_link_map.iter(), &mut inserted_items, get_key);
}

/*
 * TODO
 *      Test ideas:
 *          --  All `.get()`, `.insert()`, `.remove()` operations give the same observable
 *              behaviour as the same ones applied to `HashMap<>`.
 *          --  After all method calls, the keys in self.underlying_hash and
 *              self.order_link_map are the same.
 */
