/// The `Item` type -- represents one option to keep in the knapsack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// weight of the item, which limits what can be stored in the knapsack.
    pub weight: u32,
    /// value of the item, which decides our preference.
    pub value: u32,
}

/// The `KnapsackProblem` type -- represents the initial problem.
#[derive(Debug, Clone)]
pub struct KnapsackProblem {
    /// capacity of the knapsack -- how much the sum of Items.weight can reach.
    pub capacity: u32,
    /// options -- the list of items that we have available for carrying.
    pub options: Vec<Item>,
}

/// The `KnapsackSolution` type -- represents a filling of the knapsack.
#[derive(Debug)]
pub struct KnapsackSolution {
    /// weight of the knapsack -- sum of all Item.weight in items.
    /// this has no real purpose, it's just more conveniant to access a member than sum the
    /// items.weight values up.
    pub weight: u32,
    /// The remaining capacity in the knapsack -- the size of the knapsack originally minus the
    /// sum of items.weights.
    pub capacity: u32,
    /// The value of this knapsack -- the sum of items.value.
    /// Again, this is just for conveniance, as the information is already stored in the items
    /// vector.
    pub value: u32,
    /// The list of items stored in the knapsack in this solution.
    pub items: Vec<Item>,
}

fn vector_same_set<T: PartialEq>(left: &Vec<T>, right: &Vec<T>)
    -> bool {

    if left.len() != right.len() {
        return false;
    }

    let mut marker = vec![0; left.len()];

    for item in left {
        let mut found_item = false;
        for (index, value) in right.iter().enumerate() {
            if *value == *item && marker[index] == 0 {
                found_item    = true;
                marker[index] = 1;
                break;
            }
        }

        if !found_item {
            return false;
        }
    }

    // Apparently not possible on the stable release channel.
    // #[cfg(test)]
    // assert!(marker.iter().all(| value: &u32 | { *value == 1 }));

    return true;
}

/* Allow access to vector_same_set() from inside the test module. */
#[cfg(test)]
/// Compares the elements of two vectors.
///
/// vector_same_set() function takes two vectors of the same type that implements PartialEq and
/// returns true if they both have the same elements in them (ignoring the order).
///
/// ```
/// use knapsack_problem::vector_same_set_test;
///
/// assert!(vector_same_set_test(&vec![0, 1, 2], &vec![2, 0, 1]));
/// assert!(!vector_same_set_test(&vec![1, 1, 2], &vec![2, 1]));
/// assert!(!vector_same_set_test(&vec![0, 1, 2], &vec![3, 1, 2]));
/// ```
pub fn vector_same_set_test<T: PartialEq + Clone>(left: &Vec<T>, right: &Vec<T>)
    -> bool {
    vector_same_set(left, right)
}

impl PartialEq for KnapsackSolution {
    fn eq(&self, other: &Self) -> bool {
        let attributes_same =
            self.weight   == other.weight   &&
            self.capacity == other.capacity &&
            self.value    == other.value;

        vector_same_set(&self.items, &other.items) && attributes_same
    }
}

impl Eq for KnapsackSolution {}

/// Returns an optimal solution to the KnapsackProblem
///
/// ```
/// use rust_algorithms::knapsack_problem::{best_knapsack, KnapsackProblem, KnapsackSolution, Item};
///
/// assert_eq!(best_knapsack( KnapsackProblem {
///                               capacity: 3,
///                               options: vec![Item { weight: 10, value: 100 },
///                                             Item { weight: 1,  value: 1 }],
///                           } ),
///             KnapsackSolution {
///                 weight:   1,
///                 capacity: 2,
///                 value:    1,
///                 items:    vec![Item { weight: 1,  value: 1 }],
///             });
/// ```
pub fn best_knapsack(mut problem: KnapsackProblem)
    -> KnapsackSolution {

    if problem.options.len() == 0 {
        return KnapsackSolution {
            weight   : 0,
            value    : 0,
            capacity : problem.capacity,
            items    : problem.options
        };
    }

    let test_item    = problem.options.pop().unwrap();
    let other_items  = problem.options.clone();
    let cur_capacity = problem.capacity;

    let without_item = best_knapsack(problem);

    if cur_capacity < test_item.weight {
        without_item
    } else {
        let mut with_item = best_knapsack(KnapsackProblem {
            capacity: cur_capacity - test_item.weight,
            options: other_items,
        });
        /* TODO -- Question: what should be done in case of integer overflow? */
        if (with_item.value  +  test_item.value) > without_item.value {
            with_item.weight += test_item.weight;
            with_item.value  += test_item.value;
            with_item.items.push(test_item);
            with_item
        } else {
            without_item
        }
    }
}


#[cfg(test)]
mod tests;
