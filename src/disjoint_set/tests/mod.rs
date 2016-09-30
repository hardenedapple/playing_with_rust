#[allow(dead_code)]
use std::rc::Rc;
use std::cell::RefCell;
use disjoint_set::*;

struct ExampleNode {
    value: u8,
    node: Node,
}

fn create_node(val: u8) -> ExampleNode {
    ExampleNode {
        value: val,
        node: Rc::new(RefCell::new(Parent::Rank(0)))
    }
}

impl DisjointSet for ExampleNode {
    fn get_node(&self) -> Node {
        self.node.clone()
    }
}

#[test]
fn basic_tests() {
    let root_node = create_node(12);
    // Check that calling find() on the root node returns that very same root node.
    let full_root = root_node.find();
    match *full_root.borrow_mut() {
        Parent::Rank(rankval) => { assert_eq!(rankval, 0) },
        Parent::UpNode(_) => unreachable!(),
    };
    assert_eq!(*full_root.borrow(), *root_node.node.borrow());

    let test_node = create_node(10);
    root_node.union(&test_node);

    // Check that calling find() on the child node returns the root node.
    let child_root = test_node.find();
    match *child_root.borrow_mut() {
        Parent::Rank(rankval) => { assert_eq!(rankval, 1) },
        Parent::UpNode(_) => unreachable!(),
    };
    assert_eq!(*child_root.borrow(), *root_node.node.borrow());
}

// #[test]
// fn create_set() {
//     let test_values = vec![1u8, 254, 18, 12];
//     let basic_set = make_sets(test_values.clone());
//     for (value, node) in test_values.iter().zip(basic_set.iter()) {
//         assert_eq!(*value, node.borrow().value);
//         match *node.borrow() {
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
//     assert!(match *root.borrow() {
//         Parent::Rank(1) => true,
//         _ => unreachable!(),
//     });

//     let child: Node<u8> = basic_set[2].clone();
//     assert!(match *child.borrow() {
//         Parent::UpNode(ref x) => {
//             *x == basic_set[1]
//         },
//         _ => unreachable!(),
//     });
// }
