/*
 * Disjoint set by using the tree method.
 * In each node we have a pointer to it's parent.
 * If the current node is a root, then we have the root name.
 *
 * Order of working on this:
 *      Make a working single-threaded version of the program.
 *      Add tests for that version.
 *      Make it multi-threaded.
 *      Add tests to check the multi-threaded capabilities.
 *          (hopefully try and manually specify the order that hypothetical threads would run
 *          different functions, probably just throw a load of threads at the problem and see if
 *          anything breaks).
 *       Implement Kruskal's algorithm (as a test)
 */

/*
 * Thoughts:
 *    Work like the Solaris AVL and List implementations.
 *    The user takes a structure of their own creation, and includes a `Node`
 *    element in it.
 *    When finding the root of the set, or combining two roots, then user just
 *    passes in that element.
 *    All referencing etc is handled inside the `Node` structures, so the user
 *    doesn't have to know anything about the implementation.
 *    This would also mean the user doesn't really have to deal with creating
 *    `Nodes` or anything like that, and I can actually remove the value type in
 *    the `Node`.
 */

/*
 * In the `find()` operation, I create a stack of things that I will change on the way down.
 * This is part of the whole design of the algorithm, the Nodes on the way up all have to have
 * their Parent pointer changed to the root of this set.
 *
 * This stack must be of values that I can mutate the underlying structure with.
 *      Interior mutability (RefCell and the like)
 *      Exterior mutability (&mut or *mut)
 *      The actual values themselves (mut Node)
 *
 * The only access to the values I have in this function is from the structure that I'm following.
 * Hence, in order to have full access to the actual value, the value needs to be in the structure.
 * This is not possible for a general structure, as the compiler needs to know the size of the
 * structure at compile time.
 *      Option 3 -- NO
 *
 * If using Exterior mutability via &mut, I have the same problem -- I can
 * mutably borrow a value that is stored in the structure, but then I would need
 * the structure to contain other Nodes by value, and this is not possible.
 *      Option 2a -- NO
 *
 * If using Interior mutability -- I think I need to have an interface that takes Rc<> and
 * RefCell<> types.
 * This isn't *really* a problem, but I don't like it.
 *
 * I'll try working with raw pointers (like the rust BTree does
 * http://cglab.ca/~abeinges/blah/rust-btree-case/ ), in the near future, and see how that turns
 * out.
 *
 * For now I'll work with RefCell<> and Rc<>, and see if I can make a nice interface wrapper around
 * those.
 * Later on I'll make an alternate implementation using unsafe {} and raw
 * pointers to see how that works out.
 */

/*
 * TODO
 * Currently I have the Parent enum and the parent member of BaseNode as public.
 * This is just for tests, I don't need them as public to satisfy the API I want.
 * Hence I should read up on the dynamics of rust testing and see what I can do about removing the
 * public declarations.
 */


use std::rc::Rc;
use std::cell::RefCell;
use std::iter::FromIterator;

pub type Node<'a, T> = Rc<RefCell<BaseNode<'a, T>>>;

/// The `Parent` type -- represents a parent BaseNode or the name of the current rank.
#[derive(Debug)]
pub enum Parent<'a, T: 'a> {
    UpNode(Node<'a, T>),
    Rank(i32),
}

/// The `BaseNode` type -- represents one element in a Set.
/// It either has a parent, or it is the root element in the set, and hence represents the set as a
/// whole.
/// There are no pointers to child items, the only operation implemented is finding out what set an
/// item belongs to.
#[derive(Debug)]
pub struct BaseNode<'a, T: 'a> {
    pub parent: Parent<'a, T>,
    pub value: T,
}

// In order to allow changing of the current value
pub fn find<T>(item: Node<T>) -> Node<T> {
    if let Parent::UpNode(ref mut refcell_val) = item.borrow_mut().parent {
        let retval = find(refcell_val.clone());
        *refcell_val = retval.clone();
        return retval
    }
    item
}

pub fn make_sets<'a, T>(values: Vec<T>) -> Vec<Node<'a, T>> {
    Vec::from_iter(values.into_iter().map(
        |x|
        Rc::new(RefCell::new(BaseNode {
            parent: Parent::Rank(0),
            value: x,
        }))))
}

pub trait DisjointSet {
    fn find(self) -> Self;
    fn union(&mut self, other: &mut Self);
}

/*
 * For the look of the API I want find() to take an immutable reference to a
 * Node, and union() to take a mutable reference to the two Nodes to join.
 * I don't think it's possible to do this using the RefCell stuff: In order to
 * return an immutable reference from the find() method I need to have a
 * reference to something that exists outside of my scope. This means I have to
 * find my way through using references instead of Rc<> types (because as far as
 * Rust knows, when using Rc::clone() I'm creating something new and that will
 * go out of scope when my function returns).
 * Attempting to do that goes back to the problem I have when deciding how to
 * implement find() -- I need a mutable reference to modify the structures as I
 * pass over them, and I need some other sort of reference to follow through the
 * Nodes.
 *
 * The DisjointSet data structure need not be a Node itself -- I could have a
 * wrapper structure that contains a Node that is the Root.
 * The use of this data structure is only in finding out whether two Nodes are
 * in the same set or not, so it would be reasonably useless to work with
 * separate DisjointSet data structures.
 */
impl<'a, T> DisjointSet for Node<'a, T> {
    fn find(self) -> Self {
        if let Parent::UpNode(ref mut refcell_val) = self.borrow_mut().parent {
            let retval = refcell_val.clone().find();
            *refcell_val = retval.clone();
            return retval
        }
        self
    }


    fn union(&mut self, other: &mut Node<T>) {
    }
}


#[cfg(test)]
mod tests;
