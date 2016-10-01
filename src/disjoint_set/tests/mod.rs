use std::rc::Rc;
use std::cmp::Ordering;
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

/*
 * Implementation of Kruskal's algorithm.
 *      Have a set of edges and a set of Graph Nodes
 *      The Node structure contains an 'disjoint_set::Element' struct as a member
 *      We use this member to create disjoint sets of Nodes
 */
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

impl<'a> PartialEq for Edge<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}
impl<'a> Eq for Edge<'a> {}

impl<'a> PartialOrd for Edge<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.weight.cmp(&other.weight))
    }
}

impl<'a> Ord for Edge<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

/*
 * Eventually this will create a random graph to solve, right now it just returns a single graph
 * that I know the answer kruskals algorithm should return.
 * 
 * Creates a set of edges ordered by weight so it's easier to implement Kruskal's algorithm.
 */
macro_rules! create_graph {
    ( $nodes:ident, $edges:ident ) => {
        /*
         * Randomness nature, when I finally get round to making this actually random.
         *      - It should be possible to create a non-connected graph ( < 40% likely).
         *      - Equally likely two have an edge between any two nodes.
         *      - Weight of each edge should be random (this is easy to ensure).
         */
        let $nodes = (0..3).map(
            |x|
            Node {
                value: x,
                set_type: Rc::new(RefCell::new(ElementParent::Rank(0)))
            }).collect::<Vec<_>>();

        let mut edge_weights = vec![1, 4, 3].into_iter();
        let mut $edges = Vec::<Edge>::new();
        'outer: for (index, start) in $nodes.iter().enumerate() {
            for end in &$nodes[index+1 ..] {
                // Never going to happen at the moment -- will eventually need to be accounted for.
                let next_weight = match edge_weights.next() {
                    Some(weight) => weight,
                    None => break 'outer,
                };
                $edges.push(Edge { point_a: start, point_b: end, weight: next_weight })
            }
        }
        $edges.sort();
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
