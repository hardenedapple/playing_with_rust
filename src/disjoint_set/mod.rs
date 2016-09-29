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
 * If using Exterior mutability via &mut, I have the same problem -- I can mutably borrow a value
 * that is stored in the structure, but then I would need the structure to contain other Nodes by
 * value, and this is not possible.
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

// impl BaseNode {
//     pub fn union(&mut self, other: &mut BaseNode<T>) {
//         let (my_root, their_root = (find(self), find(other));


#[cfg(test)]
mod tests;
