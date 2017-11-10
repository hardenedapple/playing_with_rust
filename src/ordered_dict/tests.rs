use super::*;

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

    do_fetch_check(&mydict, String::from("Hello world"), 0, 10);
    do_fetch_check(&mydict, String::from("Other test"), 2, 6);
    do_fetch_check(&mydict, String::from("Test string"), 1, 15);
    do_fetch_check(&mydict, String::from("Other test"), 2, 6);

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
    for (map_item, check_item) in mydict.into_iter().zip(inserted_items) {
        assert_eq!(map_item, check_item);
    }
}

#[test]
fn iter_mut() {
    let (mut mydict, inserted_items) = create_default();
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
