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

#[derive(Debug,PartialEq,Eq)]
pub struct KnapsackSolution {
    weight: u32,
    capacity: u32,
    value: u32,
    items: Vec<Item>,
}

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
        assert!(true);
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
        let other_items = knapsack_problem.options.clone();
        let knapsack_solution = best_knapsack(knapsack_problem).unwrap();
        assert_eq!(knapsack_solution.weight, 8);
        assert_eq!(knapsack_solution.value, 15);
        assert_eq!(knapsack_solution.capacity, 7);
        for item in &other_items[1..] {
            assert!(knapsack_solution.items.contains(&item));
        }


    }

    // #[test]
    // fn order_insensitive() {
    //     let item_options = // Generate random vector
    //         ;
    // }

}
