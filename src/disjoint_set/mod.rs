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

use std::rc::Rc;
use std::cell::RefCell;

pub enum UnionResult {
    NoChange,
    Updated,
}

pub type Node<T> = Rc<RefCell<BaseNode<T>>>;

/// The `Parent` type -- represents a parent BaseNode or the name of the current rank.
#[derive(Debug)]
pub enum Parent<T: PartialEq> {
    UpNode(Node<T>),
    Rank(i32),
}

/// The `BaseNode` type -- represents one element in a Set.
/// It either has a parent, or it is the root element in the set, and hence represents the set as a
/// whole.
/// There are no pointers to child items, the only operation implemented is finding out what set an
/// item belongs to.
#[derive(Debug)]
pub struct BaseNode<T: PartialEq> {
    pub parent: Parent<T>,
}

impl<T> PartialEq for BaseNode<T> where T: PartialEq {
    fn eq(&self, other: &BaseNode<T>) -> bool {
        self as *const BaseNode<T> == other as *const BaseNode<T>
    }
}

fn find_root<T: PartialEq>(mynode: Node<T>) -> Node<T> {
    if let Parent::UpNode(ref mut refcell_val) = mynode.borrow_mut().parent {
        let retval = find_root(refcell_val.clone());
        *refcell_val = retval.clone();
        return retval
    }
    mynode
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
pub trait DisjointSet<T: PartialEq> {
    fn get_node(&self) -> Node<T>;
    fn find(&self) -> Node<T> {
        let mynode = self.get_node();
        find_root(mynode)
    }

    /*
     * I think I have a bit of a fundamental problem here.
     * In order to check whether two Nodes are in the same set, I need to check
     * whether the value I get from calling find() on them both points to the
     * same Node.
     * I can't check whether two Rc<T> values point to the same underlying
     * structure, and I hence have no idea how to implement what I need.
     *
     * Temporarily I'm using Node<T> equality to check whether the root Nodes
     * are the same.
     * This isn't really a sensible way, it doesn't proove that elements are the
     * same element, just that they are equal in some manner.
     */
    fn union(&self, other: &Self) -> UnionResult {
        let (my_root, their_root) = (self.find(), other.find());
        if my_root == their_root {
            UnionResult::NoChange
        } else {
            // TODO
            //   Make this actually neat, currently it's very ugly.
            let my_rank = match my_root.borrow().parent {
                Parent::Rank(x) => x,
                _ => unreachable!(),
            };
            let their_rank = match their_root.borrow().parent {
                Parent::Rank(x) => x,
                _ => unreachable!(),
            };
            let (greater_root, greater_rank, lesser_root) =
                if my_rank < their_rank {
                    (their_root, their_rank, my_root)
                } else {
                    (my_root, my_rank, their_root)
                };
            lesser_root.borrow_mut().parent = Parent::UpNode(greater_root.clone());
            greater_root.borrow_mut().parent = Parent::Rank(greater_rank + 1);
            UnionResult::Updated
        }
    }
}


#[cfg(test)]
mod tests;
