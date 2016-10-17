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

use std::sync::{Arc, RwLock};
use std::ops::Deref;
use std::hash::{Hash,Hasher};

pub enum UnionResult {
    NoChange,
    Updated,
}

#[derive(Debug, Clone)]
pub struct Element(Arc<RwLock<ElementParent>>);

impl Element {
    pub fn new(start_rank: i32) -> Element {
        Element(Arc::new(RwLock::new(ElementParent::Rank(start_rank))))
    }
}

impl Deref for Element {
    type Target = RwLock<ElementParent>;

    fn deref(&self) -> &RwLock<ElementParent> {
        self.0.deref()
    }
}

impl PartialEq for Element {
    /* NOTE Not 100% sure that this is sensible. It may be surprising to obtain a lock in something
     * as innocuous as an equality check. A hypothetical person using this code could get deadlock
     * without understanding why. */
    fn eq(&self, other: &Self) -> bool {
        // Using &* to take advantage of the Deref of RwLockReadGuard<>
        &*(self.read().unwrap()) as *const ElementParent ==
            &*(other.read().unwrap()) as *const ElementParent
    }
}
impl Eq for Element {}

impl DisjointSet for Element {
    fn get_node(&self) -> Element {
        self.clone()
    }
}

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
impl Eq for ElementParent {}

impl Hash for ElementParent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self as *const ElementParent).hash(state);
    }
}

fn find_root(mynode: Element) -> Element {
    if let ElementParent::UpElement(ref mut refcell_val) = *mynode.write().unwrap() {
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

    /*
     * TODO Make this multithreaded
     *  Currently there are some synchronisation problems (I hadn't written this with
     *  synchronisation in mind and just hoped it would work when I switched).
     *
     *  Between finding the roots with find() and finding the roots' Rank with match {} another
     *  thread can finish calling union() and have updated the value of either root.
     *  We would then go into a branch marked unreachable!().
     *
     *  Similarly, if another thread joins our greater_root onto a different Element before we
     *  increment its rank, we have undone that threads work joining the Elements.
     */
    fn union(&self, other: &Self) -> UnionResult {
        let (my_root, their_root) = (self.find(), other.find());
        if my_root == their_root {
            UnionResult::NoChange
        } else {
            // TODO Make this neat, currently it's very ugly.
            let my_rank = match *my_root.read().unwrap() {
                ElementParent::Rank(x) => x,
                _ => unreachable!(),
            };
            let their_rank = match *their_root.read().unwrap() {
                ElementParent::Rank(x) => x,
                _ => unreachable!(),
            };
            let (greater_root, greater_rank, lesser_root) =
                if my_rank < their_rank {
                    (their_root, their_rank, my_root)
                } else {
                    (my_root, my_rank, their_root)
                };
            *lesser_root.write().unwrap() = ElementParent::UpElement(greater_root.clone());
            *greater_root.write().unwrap() = ElementParent::Rank(greater_rank + 1);
            UnionResult::Updated
        }
    }
}


#[cfg(test)]
mod tests;
