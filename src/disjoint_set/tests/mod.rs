#[allow(dead_code)]
use std::rc::Rc;
use std::cell::RefCell;
use disjoint_set::*;

struct ExampleNode {
    value: u8,
    node: Node<u8>,
}

fn create_node(val: u8) -> ExampleNode {
    ExampleNode {
        value: val,
        node: Rc::new(RefCell::new(
            BaseNode { parent: Parent::Rank(0) }))
    }
}

impl DisjointSet<u8> for ExampleNode {
    fn get_node(&self) -> Node<u8> {
        self.node.clone()
    }
}

/*
 * TODO
 * This test isn't as complete as I'd like.
 * I want it to also check that the Rc pointers point to the same value
 * underneath, and aren't copies of anything.
 *
 * The perfect way to do this appears to be through the #![feature(ptr_eq)]
 * feature, but I haven't gotten that to work as yet.
 */

#[test]
fn basic_tests() {
    let root_node = create_node(12);
    // Check that calling find() on the root node returns that very same root node.
    let full_root = root_node.find();
    match full_root.borrow_mut().parent {
        Parent::Rank(rankval) => { assert_eq!(rankval, 0) },
        Parent::UpNode(_) => unreachable!(),
    };
    assert_eq!(full_root, root_node.node.borrow());

    let test_node = create_node(10);
    root_node.union(&test_node);

    // Check that calling find() on the child node returns the root node.
    let child_root = test_node.find();
    match child_root.borrow_mut().parent {
        Parent::Rank(rankval) => { assert_eq!(rankval, 1) },
        Parent::UpNode(_) => unreachable!(),
    };
    assert_eq!(child_root, root_node.node.borrow());
}

// #[test]
// fn create_set() {
//     let test_values = vec![1u8, 254, 18, 12];
//     let basic_set = make_sets(test_values.clone());
//     for (value, node) in test_values.iter().zip(basic_set.iter()) {
//         assert_eq!(*value, node.borrow().value);
//         match node.borrow().parent {
//             Parent::Rank(0) => {},
//             _ => unreachable!(),
//         };
//     };

//     let union_res = basic_set[1].clone().union(basic_set[2].clone());
//     assert!(match union_res {
//         UnionResult::Updated => true,
//         UnionResult::NoChange => false,
//     });

//     let root: Node<u8> = basic_set[1].clone();
//     assert!(match root.borrow().parent {
//         Parent::Rank(1) => true,
//         _ => unreachable!(),
//     });

//     let child: Node<u8> = basic_set[2].clone();
//     assert!(match child.borrow().parent {
//         Parent::UpNode(ref x) => {
//             *x == basic_set[1]
//         },
//         _ => unreachable!(),
//     });
// }
