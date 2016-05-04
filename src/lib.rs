/*
 * TODO Documentation tests -- see here
 *      https://doc.rust-lang.org/book/testing.html#documentation-tests

 * vimcmd: set makeprg=cargo\ test
 * vimcmd: !cargo test -- --ignored
 * vimcmd: !cargo test
 */

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    weight: u32,
    value: u32,
}

#[derive(Debug, Clone)]
pub struct KnapsackProblem {
    capacity: u32,
    options: Vec<Item>,
}

#[derive(Debug)]
pub struct KnapsackSolution {
    weight: u32,
    capacity: u32,
    value: u32,
    items: Vec<Item>,
}

fn vector_same_set<T: PartialEq + Clone>(left: &Vec<T>, right: &Vec<T>)
    -> bool {

    if left.len() != right.len() {
        return false;
    }

    let mut temp_right = right.to_vec();

    for item in left {
        let mut found_item = false;
        // This initialisation is not required for the consistency of the program
        // It's just here to make the compiler happy.
        // I expect that's an indication bad style, but I'll worry about that later.
        let mut position = 0;
        for (index, value) in temp_right.iter().enumerate() {
            if *value == *item { found_item = true; position = index; break; }
        }

        if found_item {
            temp_right.swap_remove(position);
        } else {
            return false;
        }
    }

    return true;
}

/* Allow access to vector_same_set() from inside the test module. */
#[cfg(test)]
pub fn vector_same_set_test<T: PartialEq + Clone>(left: &Vec<T>, right: &Vec<T>)
    -> bool {
    vector_same_set(left, right)
}

impl PartialEq for KnapsackSolution {
    fn eq(&self, other: &Self) -> bool {
        let attributes_same =
            self.weight == other.weight &&
            self.capacity == other.capacity &&
            self.value == other.value;
        vector_same_set(&self.items, &other.items) && attributes_same
    }
}

impl Eq for KnapsackSolution {}

pub fn best_knapsack(mut problem: KnapsackProblem)
    -> Result<KnapsackSolution, Vec<Item>> {

    if problem.options.len() == 0 {
        return Ok(KnapsackSolution {
            weight: 0,
            value: 0,
            capacity: problem.capacity,
            items: problem.options
        });
    }

    let test_item = problem.options.pop().unwrap();
    let other_items = problem.options.clone();
    let cur_capacity = problem.capacity;

    let without_item = best_knapsack(problem).unwrap();

    if cur_capacity < test_item.weight {
        Ok(without_item)
    } else {
        match best_knapsack(KnapsackProblem {
            capacity: cur_capacity - test_item.weight,
            options: other_items,
        }) {
            /* TODO -- Question: what should be done in case of integer overflow? */
            Ok(mut solution) =>
                if (solution.value + test_item.value) > without_item.value {
                    solution.weight += test_item.weight;
                    solution.value += test_item.value;
                    solution.items.push(test_item);
                    Ok(solution)
                } else {
                    Ok(without_item)
                },
            Err(mut err_val) => {
                err_val.push(test_item);
                Err(err_val)
            }
        }
    }
}


#[cfg(test)]
mod tests;
