use std::rc::Rc;
use std::cmp::Ordering;
use std::cell::RefCell;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::hash::{Hash,Hasher};
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
#[derive(Debug, Eq, PartialEq)]
struct Node {
    value: u32,
    set_type: Element,
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        self.set_type.borrow().hash(state);
    }
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
 * Creates a set of edges ordered by weight to facilitate Kruskal's algorithm.
 *
 * This needs to be a macro rather than a function so I can "return" a set of Node structures *and*
 * a set of Edge structures that have references to them.
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

/*
 * The vectors are of a given lifetime. This is also the lifetime of the Nodes in the first vector.
 * Because it's the lifetime of the nodes in the first vector, it's also the lifetime parameter of
 * the Edge structures.
 */
fn kruskals<'a>(nodes: &'a Vec<Node>, edges: &'a Vec<Edge<'a>>) -> Vec<&'a Edge<'a>> {
    let mut retval = Vec::new();
    let mut nodes_left: HashSet<&Node> = HashSet::from_iter(nodes);
    
    // Know that the edges are ordered by weight, so this takes the smallest weight.
    for edge in edges {
        if edge.point_a.find() == edge.point_b.find() { continue; }
        edge.point_a.union(edge.point_b);
        retval.push(edge);
        nodes_left.remove(edge.point_a);
        nodes_left.remove(edge.point_b);
        if nodes_left.is_empty() { break; }
    }

    retval
}

/*
 * Currently just checks that I got the known answer to the fixed question.
 * In the future this function actually needs to calculate some stuff.
 */
fn is_min_span_tree<'a>(nodes: &'a Vec<Node>, edges: &'a Vec<Edge<'a>>, mintree: &Vec<&'a Edge<'a>>)
    -> bool {
        mintree.len() == 2 && *mintree[0] == edges[0] && *mintree[1] == edges[1]
        // if nodes.len() <= 1
        // let maxedge = mintree.last()
}

#[test]
fn can_implement_kruskals() {
    create_graph!(nodes, edges);
    for node in &nodes {
        match *node.set_type.borrow() {
            ElementParent::Rank(0) => {},
            _ => unreachable!(),
        }
    }

    let mintree = kruskals(&nodes, &edges);
    assert!(is_min_span_tree(&nodes, &edges, &mintree));
}
