#[allow(dead_code)]
use std::rc::Rc;
use std::cell::RefCell;
use disjoint_set::*;
extern crate rand;

struct ExampleNode {
    value: u8,
    node: Element,
}

fn create_node(val: u8) -> ExampleNode {
    ExampleNode {
        value: val,
        node: Rc::new(RefCell::new(ElementParent::Rank(0)))
    }
}

impl DisjointSet for ExampleNode {
    fn get_node(&self) -> Element {
        self.node.clone()
    }
}

#[test]
fn basic_tests() {
    let root_node = create_node(12);
    // Check that calling find() on the root node returns that very same root node.
    let full_root = root_node.find();
    match *full_root.borrow_mut() {
        ElementParent::Rank(rankval) => { assert_eq!(rankval, 0) },
        ElementParent::UpElement(_) => unreachable!(),
    };
    assert_eq!(*full_root.borrow(), *root_node.node.borrow());

    let test_node = create_node(10);
    root_node.union(&test_node);

    // Check that calling find() on the child node returns the root node.
    let child_root = test_node.find();
    match *child_root.borrow_mut() {
        ElementParent::Rank(rankval) => { assert_eq!(rankval, 1) },
        ElementParent::UpElement(_) => unreachable!(),
    };
    assert_eq!(*child_root.borrow(), *root_node.node.borrow());
}

#[derive(Debug)]
struct Node {
    value: u32,
    set_type: Element,
}

#[derive(Debug)]
struct Edge<'a> {
    point_a: &'a Node,
    point_b: &'a Node,
    weight: u32,
}

impl DisjointSet for Node {
    fn get_node(&self) -> Element {
        self.set_type.clone()
    }
}

/* Eventually this will create a random graph to solve, right now it just returns a single graph
 * that I know the answer kruskals algorithm should return. */
macro_rules! create_graph {
    ( $nodes:ident, $edges:ident ) => {
        let $nodes = (0..3).map(
            |x|
            Node {
                value: x,
                set_type: Rc::new(RefCell::new(ElementParent::Rank(0)))
            }).collect::<Vec<_>>();

        let edge_weights = vec![1, 4, 3].into_iter();
        let $edges = $nodes.iter().zip(&$nodes).zip(edge_weights).map(
            |((a, b), weight)|
            Edge { point_a: a, point_b: b, weight: weight }
            ).collect::<Vec<_>>();
    };
}

#[test]
fn create_set() {
    create_graph!(nodes, edges);
    for node in &nodes {
        match *node.set_type.borrow() {
            ElementParent::Rank(0) => {},
            _ => unreachable!(),
        }
    }

    let union_res = nodes[1].union(&nodes[2]);
    assert!(match union_res {
        UnionResult::Updated => true,
        UnionResult::NoChange => false,
    });

    let root: Element = nodes[1].find();
    assert!(match *root.borrow() {
        ElementParent::Rank(1) => true,
        _ => unreachable!(),
    });

    let other_root: Element = nodes[2].find();
    assert_eq!(other_root, root);
}
