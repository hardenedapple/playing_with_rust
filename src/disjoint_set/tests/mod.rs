use std::rc::Rc;
use std::cmp::Ordering;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap, VecDeque};
use std::hash::{Hash,Hasher};
use disjoint_set::*;
use test_utils::*;
extern crate rand;


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
 * Eventually this will create a random graph to solve, right now it just returns a single graph
 * that I know the answer kruskals algorithm should return.
 *
 * Creates a set of edges ordered by weight to facilitate Kruskal's algorithm.
 *
 * This needs to be a macro rather than a function so I can "return" a set of Node structures *and*
 * a set of Edge structures that have references to them.
 *
 * TODO Make the random properties of this function more like what I want.
 * Intended Randomness nature:
 *      [X] Possible to create a non-connected graph ( < 40% likely).
 *      [ ] Equally likely to have an edge between any two nodes.
 *      [X] Weight of each edge should be random (this is the easiest)
 */
macro_rules! create_graph {
    ( $nodes:ident, $edges:ident ) => {
        let $nodes = (0..rand::random::<u8>()).map(
            |x| create_node(x as u32)).collect::<Vec<_>>();

        // From how I create the edges, if the length of the weight vector is
        // ($nodes.len() - 1) or greater then an MST is possible, and otherwise
        // it's not.
        // Hence, for the moment, setting our vector to a random length less
        // than (3 * $nodes.len()) means that we are less than 40% likely to
        // make a graph that is connected. (because 1/0.4 == 2.5, and the
        // difference of the - 2 and our actual multiple moves things in the
        // direction that make it more likely for our graph to be connected).
        let mut edge_weights = random_vector(3 * $nodes.len()).into_iter();
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

        // Store this edge of the MST into the tree so far -- (next iteration
        // will be over the edges in the graph greater than this edge in the MST
        // and smaller than the next edge).
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
        Ok(ref tree) => assert!(is_min_span_tree(&edges, &tree)),
        Err(_) => assert!(cant_make_join(&nodes, &edges)),
    }
}
