/*
 * vimcmd: set makeprg=cargo\ test
 * vimcmd: !cargo test
 */

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Item {
    weight: u32,
    value: u32,
}

#[derive(Debug)]
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

    let mut temp_left = left.to_vec();
    let mut temp_right = left.to_vec();

    for item in temp_left.drain(..) {
        let mut found_item = false;
        // This initialisation is not required for the consistency of the program
        // It's just here to make the compiler happy.
        // I expect that's an indication bad style, but I'll worry about that later.
        let mut position = 0;
        for (index, value) in temp_right.iter().enumerate() {
            if *value == item { found_item = true; position = index; break; }
        }

        if found_item {
            temp_right.swap_remove(position);
        } else {
            return false;
        }
    }

    return true;
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

    let without_item = best_knapsack( KnapsackProblem { .. problem }).unwrap();

    if problem.capacity < test_item.weight {
        Ok(without_item)
    } else {
        match best_knapsack(KnapsackProblem {
            capacity: problem.capacity - test_item.weight,
            options: other_items,
        }) {
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
mod tests {
    use super::*;
    #[test]
    fn it_works() {
    }

    #[test]
    fn handles_base_case() {
        let knapsack_problem = KnapsackProblem { capacity: 0, options: Vec::new() };
        let knapsack_solution = best_knapsack(knapsack_problem).unwrap();
        let correct_solution = KnapsackSolution {
            weight: 0, value: 0, capacity: 0, items: Vec::new(),
        };
        assert_eq!(knapsack_solution, correct_solution);
    }

    #[test]
    fn handles_simple() {
        let item_options = vec![
            Item { weight: 12, value: 4 },
            Item { weight: 1, value: 1 },
            Item { weight: 1, value: 2 },
            Item { weight: 2, value: 2 },
            Item { weight: 4, value: 10 }
        ];

        let knapsack_problem = KnapsackProblem {
            capacity: 15,
            options: item_options.clone(),
        };
        let correct_solution = KnapsackSolution {
            weight: 8,
            value: 15,
            capacity: 7,
            items: item_options[1..].to_vec(),
        };

        assert_eq!(best_knapsack(knapsack_problem).unwrap(), correct_solution);
    }

    // #[test]
    // fn order_insensitive() {
    //     let item_options = // Generate random vector
    //         ;
    //     let mut all_permutations = item_options.permutations();
    //     let first_solution = all_permutations.next();
    //     for solution in all_permutations {
    //         assert_eq!(solution.weight, 8);
    //         assert_eq!(solution.value, 15);
    //         assert_eq!(solution.capacity, 7);
    //         for item in &other_items[1..] {
    //             assert!(solution.items.contains(&item));
    //         }
    //     }
    // }

}
