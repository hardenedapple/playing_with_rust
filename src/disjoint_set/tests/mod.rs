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
fn kruskals<'a>(nodes: &'a Vec<Node>, edges: &'a Vec<Edge<'a>>)
  -> Result<Vec<&'a Edge<'a>>, Vec<&'a Edge<'a>>> {
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

    if nodes_left.is_empty() {
        Ok(retval)
    } else {
        Err(retval)
    }
}

/*
 * Currently just checks that I got the known answer to the fixed question.
 * In the future this function actually needs to calculate some stuff.
 */
fn is_min_span_tree<'a>(nodes: &'a Vec<Node>, edges: &'a Vec<Edge<'a>>, mintree: &Vec<&'a Edge<'a>>)
    -> bool {
        mintree.len() == 2 && *mintree[0] == edges[0] && *mintree[1] == edges[1]
        // if nodes.len() <= 1 {
        //     mintree.len() == 1
        // } else {
        //     let maxedge = mintree.last()
        //     // mintree is a MST iff for every edge (u, v) not in mintree, the path between u and
        //     // v in mintree
        // }
}

fn my_split_at<'a>(target: &'a Edge<'a>, edges: &'a [Edge<'a>])
    -> Option<(&'a [Edge<'a>], &'a [Edge<'a>])> {

    let mut start_index = None;
    for (index, edge) in edges.iter().enumerate() {
        if edge == target && start_index.is_none() { start_index = Some(index); }
        if edge as *const Edge == target as *const Edge { break; }
        if edge > target { return None; }
    }

    match start_index {
        Some(index) => {
            let (smaller, rest) = edges.split_at(index);
            Some((smaller, rest))
        },
        None => None,
    }
}

/*
 * TODO
 *      I believe this function checks if something is a minimum spanning tree by a reverse of
 *      kruskals algorithm.
 *          Proove this.
 */
fn no_missing_edges<'a>(edges: &'a Vec<Edge<'a>>, mintree: &'a Vec<&'a Edge<'a>>) -> bool {
    /*
     * Find set difference of edges and mintree (elements in edges not in mintree, all elements in
     * mintree should be in edges).
     * For each edge in this difference, unless both Nodes are part of an Edge in mintree, return
     * false.
     * Otherwise, return true.
     */
    let mut observed_nodes = HashSet::new();
    let (_, mut remaining_edges) = edges.split_at(0);
    let mut smaller_edges: &[Edge];

    /*
     * Use the fact that both edges and mintree are ordered by weight to assert some invariants
     * like when an edge is not taken then both nodes should already be in the graph.
     */
    'outer: for tree_edge in mintree.into_iter() {
        // If both of these nodes are already in the MST candidate, then this edge is superfluous.
        let mut either_node_new = observed_nodes.insert(tree_edge.point_a);
        either_node_new |= observed_nodes.insert(tree_edge.point_b);
        if !either_node_new {
            return false;
        }

        /*
         * my_split_at() returns edges smaller than the target, and a list of all edges of greater
         * or equal weight to the target.
         *
         * NOTE:
         *      Rather than jump through a bunch of hoops to avoid it, I'm performing a superfluous
         *      check that edges which are contained in the MST connect two points that are
         *      connected by the MST.
         *      I intend to have a little think about this when I'm more familiar with Rust to see
         *      if there's anything nice I can do to "fix" this.
         */
        let (left, right) = match my_split_at(tree_edge, remaining_edges) {
            Some((left, right)) => (left, right),
            None => return false,
        };
        smaller_edges = left;
        remaining_edges = right;

        for small_edge in smaller_edges {
            // This edge is not in the candidate MST, and has a smaller weight than any of
            // the edges we haven't accounted for in the MST so far.
            // The nodes it connects must have already been seen (by a reverse of the
            // argument that proves kruskals algorithm).
            //
            // Also, there must be a path between point_a and point_b in the nodes seen at
            // the moment. This is the condition for an MST.
            //  TODO -- I don't check this
            if !(observed_nodes.contains(small_edge.point_a) &&
                 observed_nodes.contains(small_edge.point_b)) {
                return false;
            }
        }
    }

    /*
     * Check that all other edges connect nodes that are already in the MST.
     */
    for graph_edge in edges  {
        // This edge is not in the candidate MST.
        // The nodes it connects must already be there (if implementing kruskals algorithm).
        if !(observed_nodes.contains(graph_edge.point_a) &&
             observed_nodes.contains(graph_edge.point_b)) {
            return false;
        }
    }

    true
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
    match mintree {
        Ok(ref tree) => assert!(is_min_span_tree(&nodes, &edges, &tree)),
        Err(ref tree) => assert!(no_missing_edges(&edges, &tree)),
    }
}
