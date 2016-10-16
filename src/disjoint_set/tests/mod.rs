use std::rc::Rc;
use std::cmp::Ordering;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap, VecDeque};
use std::hash::{Hash,Hasher};
use disjoint_set::*;
use test_utils::*;
extern crate rand;


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

fn create_node(val: u32) -> Node {
    Node {
        value: val,
        set_type: Rc::new(RefCell::new(ElementParent::Rank(0)))
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
    assert_eq!(*full_root.borrow(), *root_node.set_type.borrow());

    let test_node = create_node(10);
    root_node.union(&test_node);

    // Check that calling find() on the child node returns the root node.
    let child_root = test_node.find();
    match *child_root.borrow_mut() {
        ElementParent::Rank(rankval) => { assert_eq!(rankval, 1) },
        ElementParent::UpElement(_) => unreachable!(),
    };
    assert_eq!(*child_root.borrow(), *root_node.set_type.borrow());
}

/*
 * Creates a set of edges ordered by weight to facilitate Kruskal's algorithm.
 *
 * This needs to be a macro rather than a function so I can "return" a set of Node structures *and*
 * a set of Edge structures that have references to them.
 */
macro_rules! create_graph {
    ( $nodes:ident, $edges:ident ) => {
        // Max of 100 elements in order to avoid taking too long.
        let $nodes = (0..(rand::random::<u32>() % 100)).map(
            |x| create_node(x)).collect::<Vec<_>>();

        let mut edge_weights = random_vector(10 * $nodes.len());
        edge_weights.sort();
        /*
         * TODO
         *      I've found by sampling that this number of elements seems to give me a probability
         *      of a connected graph of just under 4:1.
         *      I'd like to find the actual mathematical probability of connectedness given these
         *      parameters.
         *
         *  The question at
         *  http://math.stackexchange.com/questions/584228/exact-probability-of-random-graph-being-connected
         *  gives the formula for the probability that a graph is connected G(n, p) where 'p' is
         *  the probability of any two edges and 'n' is the number of nodes.
         *  That question ignores self-edges, but I don't think that matters -- as long as the
         *  probability of edges between nodes that aren't the same is calculated properly it
         *  doesn't change any of the reasoning presented in that answer.
         *  The probability that any two Nodes are joined with an edge when using the method I'm
         *  using to create edges is 1 - ((n**2 - 2) / n**2)**N where 'N' is the number of
         *  iterations.
         *      The probability two particular nodes are joined in a given order for each edge is
         *      1/N**2.
         *      In order to allow both directions we multiply by 2.
         *      The probability we don't join these two edges is then 1 - (2 / N**2) = 
         *      (N**2 - 2) / N**2 = X.
         *      The probability we haven't joined those two edges in 'n' iterations is X**N.
         *      Hence the probability of two Nodes being connected after 'N' iterations is
         *      1 - X**N.
         *
         * Putting that probability into the equation we get from the web page above just changes
         * the term (1 - p)**(i(n - i)) to (X**N)**(i(n-i)).
         * This doesn't give me a nice equation that I can just plug the numbers into, it gives me
         * a recursion relation.
         *
         * From tests, I know that the relation is not linear, nor quadratic.
         * In order to run the tests, take length to be max_length in random_vector() instead of
         * having a random value, then instead of choosing a random number of nodes set it to a
         * specific value.
         * Once these changes have been made, you can create a graph multiple times, changing how
         * many times a graph is created in can_implement_kruskals() and changing how many nodes
         * each graph has above.
         *
         * The ratio of connected graphs to unconnected graphs is stored below.
         *
         * Num weights == 2 * Num Nodes
         * 10 Nodes   4.30, 4.38, 4.298
         * 100 Nodes  0.155, 0.164, 0.157
         * 1000 Nodes  0, 0, 0
         *
         * Num weights = (Num Nodes)**40
         * 10 Nodes    0, 0, 0
         * 100 Nodes   0.8797, 0.852, 1.0
         * 1000 Nodes  inf, inf
         *
         * It's much closer to a linear relation than a quadratic one, but it's not linear.
         */

        /* NOTE -- This allows multiple edges between the same two Nodes, it's not a problem. */
        let mut $edges = Vec::<Edge>::new();
        for weight in edge_weights.into_iter() {
            $edges.push(Edge {
                point_a: &$nodes[rand::random::<usize>() % $nodes.len()],
                point_b: &$nodes[rand::random::<usize>() % $nodes.len()],
                weight: weight
            })
        }
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

    // Know that the edges are ordered by weight, so this takes the smallest weight.
    for edge in edges {
        if edge.point_a.find() == edge.point_b.find() { continue; }
        edge.point_a.union(edge.point_b);
        retval.push(edge);
    }

    if nodes.iter().all(|x| x.find() == nodes[0].find()) {
        Ok(retval)
    } else {
        Err(retval)
    }
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

fn path_exists<'a>(current_mst: &'a HashMap<&'a Node, HashSet<&'a Node>>,
                   start: &'a Node, end: &'a Node) -> bool {
    let mut inspect_elements = VecDeque::new();;
    let mut seen_elements = HashSet::new();

    inspect_elements.push_back(start);
    seen_elements.insert(start);


    while let Some(next_element) = inspect_elements.pop_front() {
        if next_element == end { return true; }
        if let Some(adj_list) = current_mst.get(next_element) {
            inspect_elements.extend(adj_list.iter()
                                    .filter(|x| seen_elements.insert(x)));
        }
    }

    false
}

macro_rules! add_to_adjacency {
    ($edge:expr, $partial_mst:expr) => {
        for (node, othernode) in vec![($edge.point_a, $edge.point_b),
                                      ($edge.point_b, $edge.point_a)].into_iter() {
            // TODO -- is there some pretty way of checking this?
            //    if let takes the mutable reference for the entire if
            //    expression (including the else branch) which means I can't
            //    borrow partial_mst mutably in the else branch when I want to
            //    insert a new adjacency list.
            let mut flag = false;
            if let Some(mut set) = $partial_mst.get_mut(node) {
                set.insert(othernode);
                flag = true;
            }
            if flag == false {
                let mut temp_adjacency = HashSet::new();
                temp_adjacency.insert(othernode);
                $partial_mst.insert(node, temp_adjacency);
            }
        }
    }
}

fn is_min_span_tree<'a>(edges: &'a Vec<Edge<'a>>, mintree: &'a Vec<&'a Edge<'a>>) -> bool {
    /*
     * Find set difference of edges and mintree (elements in edges not in mintree, all elements in
     * mintree should be in edges).
     * For each edge in this difference, unless both Nodes are part of an Edge in mintree, return
     * false.
     * Otherwise, return true.
     */
    let mut partial_mst: HashMap<&Node, HashSet<&Node>> = HashMap::new();
    let (_, mut remaining_edges) = edges.split_at(0);
    let mut smaller_edges: &[Edge];

    /*
     * Use the fact that both edges and mintree are ordered by weight to assert some invariants
     * like when an edge is not taken then both nodes should already be in the graph.
     */
    'outer: for tree_edge in mintree.into_iter() {
        /*
         * my_split_at() returns edges smaller than the target, and a list of all edges of greater
         * or equal weight to the target.
         *
         * NOTE:
         *      Rather than jump through a bunch of hoops to avoid it, I'm
         *      checking that a path exists (below) even for edges that are in
         *      the MST so far.
         */
        let (left, right) = match my_split_at(tree_edge, remaining_edges) {
            Some((left, right)) => (left, right),
            None => return false,
        };
        smaller_edges = left;
        remaining_edges = right;

        for small_edge in smaller_edges {
            /*
             * The condition for something to be an MST is that the path between
             * each pair of nodes connected by any edge in the graph must consist
             * only of branches smaller or equal in weight to that edge.
             * The edge we are currently looking at has a smaller weight than
             * all branches in our candidate MST that are not in partial_mst,
             * and a larger weight than those branches that are in partial_mst.
             * Hence, in order to proove that our MST is correct, it is
             * sufficient to show is that the nodes this edge connects already
             * have a path between them in partial_mst.
             */
            if !path_exists(&partial_mst, &small_edge.point_a, &small_edge.point_b) {
                return false;
            }
        }

        /*
         * Store this edge of the MST into the tree so far -- (next iteration
         * will be over the edges in the graph greater than this edge in the MST
         * and smaller than the next edge).
         */
        add_to_adjacency!(tree_edge, partial_mst);
    }

    /*
     * Check that all other edges connect nodes that are already in the MST.
     */
    for graph_edge in edges  {
        // This edge is not in the candidate MST.
        // The nodes it connects must be connected by the MST.
        if !path_exists(&partial_mst, &graph_edge.point_a, &graph_edge.point_b) {
            return false;
        }
    }

    true
}

fn cant_make_join<'a>(nodes: &'a Vec<Node>, edges: &'a Vec<Edge<'a>>) -> bool {
    // Check no edges connect two disjoint sets.
    let mut partial_mst: HashMap<&Node, HashSet<&Node>> = HashMap::new();
    for edge in edges {
        assert!(edge.point_a.find() == edge.point_b.find());
        add_to_adjacency!(edge, partial_mst);
    }

    let mut retval = false;

    // Assert that there is some node not connected to another node.
    let first_set = nodes[0].find();
    for node in nodes {
        if first_set != node.find() {
            // Here we check there is actually no connection (without relying on
            // the disjoint_set implementation that we're testing).
            assert!(!path_exists(&partial_mst, &nodes[0], node));
            retval = true;
        }
    }

    return retval;
}

#[test]
#[ignore]
fn can_implement_kruskals() {
    for _ in 0..500 {
        create_graph!(nodes, edges);
        match kruskals(&nodes, &edges) {
            Ok(ref tree) => assert!(is_min_span_tree(&edges, &tree)),
            Err(_) => assert!(cant_make_join(&nodes, &edges))
        }
    }
}
