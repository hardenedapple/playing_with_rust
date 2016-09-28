#[allow(dead_code)]
use std::rc::Rc;
use std::cell::RefCell;
use disjoint_set::*;


fn make_singly_wrapped<'a, T>(item: Node<'a, T>) -> SinglyWrapped<'a, T> {
    Rc::new(RefCell::new(item))
}

fn make_doubly_wrapped<'a, T>(item: Node<'a, T>) -> DoublyWrapped<'a, T> {
    RefCell::new(make_singly_wrapped(item))
}

/*
 * TODO
 * This test isn't as complete as I'd like.
 * I want it to also check that the Rc pointers point to the same value underneath, and aren't
 * copies of anything.
 *
 * The perfect way to do this appears to be through the #![feature(ptr_eq)] feature, but I haven't
 * gotten that to work as yet.
 */

#[test]
fn basic_tests() {
    let (xval, yval) = (&12, &400);
    let root_node = Node {
        parent: Parent::Rank(12),
        value: xval,
    };
    // Check that calling find() on the root node returns that very same root node.
    let full_root = find(make_singly_wrapped(root_node));
    match full_root.borrow_mut().parent {
        Parent::Rank(rankval) => { assert_eq!(rankval, 12) },
        Parent::UpNode(_) => unreachable!(),
    };

    let test_node = Node { 
        parent: Parent::UpNode(RefCell::new(full_root.clone())),
        value: yval,
    };

    // Check that calling find() on the child node returns the root node.
    let child_root = find(make_singly_wrapped(test_node));
    match child_root.borrow_mut().parent {
        Parent::Rank(rankval) => { assert_eq!(rankval, 12) },
        Parent::UpNode(_) => unreachable!(),
    };
}
