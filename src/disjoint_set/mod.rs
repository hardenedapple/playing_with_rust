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

/*
 * NOTE
 *      Need this so that we can view the next "step" we follow without immutably borrowing
 *      the vector for the lifetime of the value we read.
 *      I need the "lifetime" of next_parent to be as long as the lifetime of guard_vector
 *      so that I can store a reference to it in that vector.
 *      As far as Rust knows, I could dereference the next_parent value after guard_vector
 *      has reallocated, and be dereferencing freed memory.
 *      I don't believe this is the case because of how I access things (though I'm not
 *      100% sure moving an Arc<> type doesn't affect the RwLockWriteGuard<> taken from the
 *      RwLock<> inside that Arc<>).
 */
macro_rules! last_ele_unbound {
    ($vector:expr) => {
        unsafe { &*($vector.last().unwrap().deref() as *const ElementParent) }
    } 
}

macro_rules! find_locked {
/* TODO Do I need to keep the lock while following the chain?
 *      There are two options -- have a small gap between dropping the lock on the "current"
 *      and getting the lock on the "next", or not.
 *
 *      No gap Case 1:
 *          Two processes, disjoint set of three Nodes   A -> B -> C
 *          Run find on A at the same time, don't hold any lock while searching higher Nodes.
 *          First process gets a lock on A, second process can't do anything.
 *          First process drops lock on A, gets lock on B
 *          Second process can get lock on A, but can't get past B.
 *          This carries on until first process gets lock on C and attempts to get lock on B to
 *          modify it on the way down.
 *          At this point, the second process can't get lock on C (because the whole point is
 *          to keep a lock on C for the return), and hence doesn't drop the lock on B.
 *          DEADLOCK!
 *
 *      Gap Case 1:
 *          Following the above until the first process attempts to lock B for the second time,
 *          at this point it gets it because the second process has dropped lock B in
 *          preparation for obtaining lock C.
 *
 *      If only get read lock on each Node when following path, then have to upgrade lock once
 *      reach root.
 *      This would introduce a race, and you'd have to check that nothing changed during
 *      dropping the read lock and obtaining the lock.
 *
 *      I doubt it's worth the extra effort coding (especially as there would probably be much more
 *      contention if locks are being dropped & obtained all over the place).
 */
    ($init_node:expr, $inner_parent:ident, $inner_guard:ident) => {

        /* NOTE -- remember drop() order at end of fn here */
        let $inner_parent; // Lifetime of innermost parent must exceed lifetime of guard
        let $inner_guard; // Want RwLockWriteGuard to last until end of block (the whole point)
        {
            let mut parent_vector = vec![$init_node.clone()];
            let mut guard_vector = vec![$init_node.write().unwrap()];
            loop {
                if let ElementParent::UpElement(ref next_parent) = *last_ele_unbound!(guard_vector) {
                    // TODO
                    //      If guard_vector.push() reallocates, and hence the reference to next_parent is
                    //      no longer valid, does that mean the lock I have obtained here is also no longer
                    //      valid?
                    //      Can test for reallocation by using with_capacity(), capacity(), and
                    //      shrink_to_fit(), but what can I look for to tell whether the guard has been
                    //      invalidated.
                    //
                    //      Justification/Reasoning/This code is just a play around anyway:
                    //          The Arc<> data type that I'm storing conceptually owns the RwLock<> data
                    //          type, but the actual data isn't there.
                    //          When the vector reallocates, it will move the Arc<> structure, but have no
                    //          affect on the RwLock<> structure that is stored elsewhere.
                    //          The RwLockWriteGuard<> is not invalidated as it has no requirement on where
                    //          the Arc<> data structures are.
                    //
                    //          I think it unlikely that the Arc<> types will reach into whatever they own
                    //          in order to invalidate them.

                    parent_vector.push(next_parent.clone());
                    // Use temporary variable just to make 100% certain of the order between next_parent
                    // being dereferenced and guard_vector being modified.

                    /* If the current thread already has the write lock on this parent, then we
                     * can't get it again. Test this by looking if the ElementParent under this
                     * RwLock is the same as the other ElementParent we have (only ever use this
                     * macro in union()) */
                    let temp_guard = next_parent.write().unwrap();
                    guard_vector.push(temp_guard);
                } else {
                    $inner_parent = parent_vector.pop().unwrap();
                    $inner_guard = guard_vector.pop().unwrap();
                    break;
                }
            }

            for mut guard in guard_vector.into_iter().rev() {
                *guard = ElementParent::UpElement($inner_parent.clone());
            }

            /* Shouldn't be needed, but nice little check */
            for parent in parent_vector.into_iter() {
                match parent.try_write() {
                    Ok(_) => {},
                    Err(_) => panic!("Outer values are locked"),
                }
            }
        }

        /* Shouldn't be needed, but nice little check */
        match $inner_parent.try_write() {
            Ok(_) => panic!("Inner value is not locked!"),
            Err(_) => {},
        };
    }
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
     *
     *  If this were a different language, I could write another function find_locked() that
     *  returns from its recursion still holding the write lock on the root node.
     *  I could then change the pointers around under these locks in union(), then release them
     *  once done.
     *  I'm having a lot of trouble doing this in Rust. 
     *  I want to have the RwLockWriteGuard<ElementParent> of the root around while I'm jiggling
     *  pointers and values.
     *  Because the RwLock::write() method takes a &self, I hence need to keep the root structure
     *  around too.
     */

    fn union(&self, other: &Self) -> UnionResult {
        let my_init = self.get_node();
        let their_init = other.get_node();
        find_locked!(my_init, my_root, my_guard);
        find_locked!(their_init, their_root, their_guard);
        // This can't be the case at the moment because the second find_locked!() would have
        // panic'd -- in the future I'll make something so that that macro instead tells this block
        // that something has happened.
        assert!(my_guard.deref() as *const ElementParent !=
                their_guard.deref() as *const ElementParent);

        // TODO Make this neat, currently it's very ugly.
        let my_rank = match *my_guard {
            ElementParent::Rank(x) => x,
            _ => unreachable!(),
        };
        let their_rank = match *their_guard {
            ElementParent::Rank(x) => x,
            _ => unreachable!(),
        };
        let (greater_root, mut greater_guard, greater_rank, mut lesser_guard) =
            if my_rank < their_rank {
                (their_root, their_guard, their_rank, my_guard)
            } else {
                (my_root, my_guard, my_rank, their_guard)
            };
        *lesser_guard = ElementParent::UpElement(greater_root.clone());
        *greater_guard = ElementParent::Rank(greater_rank + 1);
        UnionResult::Updated
    }
}

#[cfg(test)]
fn attempt_find_locked(init_node: Element) -> bool {
    find_locked!(init_node, inner_parent, inner_guard);
    let retval = match inner_parent.try_write() {
        Ok(_) => panic!("Inner value is not locked!"),
        Err(_) => true,
    };
    retval
}



#[test]
fn three_deep_find_locked() {
    let elements = (0..3).map(Element::new).collect::<Vec<_>>();
    let element0 = elements[0].clone();
    let element1 = elements[1].clone();
    let element2 = elements[2].clone();
    *element0.write().unwrap() = ElementParent::UpElement(element1.clone());
    *element1.write().unwrap() = ElementParent::UpElement(element2.clone());

    assert!(attempt_find_locked(elements[0].clone()));
    assert_eq!(elements.len(), 3);
}

#[cfg(test)]
mod tests;
