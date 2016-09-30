/*
 * In the `find()` operation, I create a stack of things that I will change on the way down.
 * This is part of the whole design of the algorithm, the Elements on the way up all have to have
 * their ElementParent pointer changed to the root of this set.
 *
 * This stack must be of values that I can mutate the underlying structure with.
 *      Interior mutability (RefCell and the like)
 *      Exterior mutability (&mut or *mut)
 *      The actual values themselves (mut Element)
 *
 * The only access to the values I have in this function is from the structure that I'm following.
 * Hence, in order to have full access to the actual value, the value needs to be in the structure.
 * This is not possible for a general structure, as the compiler needs to know the size of the
 * structure at compile time.
 *      Option 3 -- NO
 *
 * If using Exterior mutability via &mut, I have the same problem -- I can
 * mutably borrow a value that is stored in the structure, but then I would need
 * the structure to contain other Elements by value, and this is not possible.
 *      Option 2a -- NO
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

pub type Element = Rc<RefCell<ElementParent>>;

/// The `ElementParent` type -- represents a Element or the name of the current rank.
#[derive(Debug)]
pub enum ElementParent {
    UpElement(Element),
    Rank(i32),
}

impl PartialEq for ElementParent {
    fn eq(&self, other: &ElementParent) -> bool {
        self as *const ElementParent == other as *const ElementParent
    }
}

fn find_root(mynode: Element) -> Element {
    if let ElementParent::UpElement(ref mut refcell_val) = *mynode.borrow_mut() {
        let retval = find_root(refcell_val.clone());
        *refcell_val = retval.clone();
        return retval
    }
    mynode
}

pub trait DisjointSet {
    fn get_node(&self) -> Element;

    fn find(&self) -> Element {
        let mynode = self.get_node();
        find_root(mynode)
    }

    fn union(&self, other: &Self) -> UnionResult {
        let (my_root, their_root) = (self.find(), other.find());
        if my_root == their_root {
            UnionResult::NoChange
        } else {
            // TODO
            //   Make this neat, currently it's very ugly.
            let my_rank = match *my_root.borrow() {
                ElementParent::Rank(x) => x,
                _ => unreachable!(),
            };
            let their_rank = match *their_root.borrow() {
                ElementParent::Rank(x) => x,
                _ => unreachable!(),
            };
            let (greater_root, greater_rank, lesser_root) =
                if my_rank < their_rank {
                    (their_root, their_rank, my_root)
                } else {
                    (my_root, my_rank, their_root)
                };
            *lesser_root.borrow_mut() = ElementParent::UpElement(greater_root.clone());
            *greater_root.borrow_mut() = ElementParent::Rank(greater_rank + 1);
            UnionResult::Updated
        }
    }
}


#[cfg(test)]
mod tests;
