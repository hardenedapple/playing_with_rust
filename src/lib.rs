/*
 * vimcmd: set makeprg=cargo\ test
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

pub fn vector_same_set<T: PartialEq + Clone>(left: &Vec<T>, right: &Vec<T>)
    -> bool {

    if left.len() != right.len() {
        return false;
    }

    let mut temp_left = left.to_vec();
    let mut temp_right = right.to_vec();

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
            /* TODO -- Question, what should be done in case of integer overflow? */
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
    extern crate rand;
    use self::rand::Rng;
    const MAX_VECTOR_SIZE: usize = 30;

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

    /*
     * NOTE:
     *  This function returns Items with attributes in the 16 bit range.
     *  This is so that with 30 elements in a knapsack, it should be impossible for the value or
     *  weight to overflow a 32 bit unsigned integer.
     */
    impl rand::Rand for Item {
        fn rand<R: rand::Rng>(rng: &mut R) -> Item {
            let tuple: (u16, u16) = rng.gen();
            Item { weight: tuple.0 as u32, value: tuple.1 as u32 }
        }
    }

    /* Thoughts on what might be interesting:
     * If I make the distribution less random and more weighted to the edges, this may catch more
     * edge cases in the code.
     * See https://doc.rust-lang.org/rand/rand/distributions/gamma/index.html
     * https://doc.rust-lang.org/rand/rand/distributions/index.html
     */
    fn random_vector<T: rand::Rand>(max_length: usize)
        -> Vec<T> {
            if max_length == 0 {
                return Vec::new();
            }

            let length: usize = rand::random::<usize>() % max_length;
            let mut retval: Vec<T> = Vec::with_capacity(length);
            let mut rng = rand::thread_rng();
            for _ in 0..length {
                retval.push(rng.gen());
            }
            retval
    }

    #[test]
    fn same_result_each_time() {
        let rand_vec = random_vector(rand::random::<usize>() %
                                               MAX_VECTOR_SIZE);
        let random_capacity = rand::random::<u32>();

        let original_answer = best_knapsack(KnapsackProblem {
            capacity: random_capacity,
            options: rand_vec.clone(),
        });
        let other_answer = best_knapsack(KnapsackProblem {
            capacity: random_capacity,
            options: rand_vec.clone(),
        });

        assert_eq!(other_answer, original_answer);
    }

    #[allow(dead_code)]
    struct VectorPermutations<T> {
        permutation: Vec<T>,
        count: Vec<u32>,
        started: bool,
        index: u32,
    }

    // impl Iterator for VectorPermutations<T> {
    //     type Item = &Vec<T>;

    //     // // Implement Heap's algorithm for permutations
    //     // fn next(&mut self) -> Option<&Vec<T>> {
    //     //     // Start and end conditions
    //     //     let num_elements = self.permutation.len();
    //     //     if !self.started {
    //     //         if permutation.is_empty() {
    //     //             return None;
    //     //         }
    //     //         self.started = true;
    //     //         // Return original permutation as first one.
    //     //         Some(&self.permutation)
    //     //     } else if self.index == num_elements {
    //     //         // This is our finished condition
    //     //         None
    //     //     } else {
    //     //         // Move one more step along Heap's algorithm
    //     //         if self.count[self.index] < self.index {
    //     //         }
    //     //     }
            
    //     }
    // }

    #[allow(dead_code)]
    impl <T> VectorPermutations<T> {
        fn from_vec(original_vector: Vec<T>)
            -> VectorPermutations<T> {
                let len = original_vector.len();
                VectorPermutations {
                    permutation: original_vector,
                    count: vec![0; len],
                    started: false,
                    index: 0,
                }
        }
    }

    // #[test]
    // fn order_insensitive() {
    //     let max_weight = rand::random();
    //     let max_value = rand::random();
    //     let max_length = rand::random() % 10;
    //     let item_options = random_item_vector(max_weight, max_value, max_length);
    //     /* 
    //      * Have a random set of Items.
    //      * Create an iterator over each permutation of items.
    //      * For each permutation, check that my function gives the same result.
    //      */
    //     let mut permutations = VectorPermutations::from_vec(item_options);
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

    fn alternate_same_set<T: PartialEq + Ord>(left: &mut Vec<T>, right: &mut Vec<T>) -> bool {
        left.sort(); right.sort();
        left == right
    }

    #[test]
    fn set_fail_comparison() {
        let left = vec![false, true];
        let right = vec![false, false];
        assert!(!vector_same_set(&left, &right));
    }

    #[test]
    fn set_comparison() {
        let mut left: Vec<bool> = random_vector(MAX_VECTOR_SIZE);
        let vector_length = left.len();
        let mut right = left.clone();
        if vector_length != 0 {
            right[rand::random::<usize>() % vector_length] = rand::random::<bool>();
        }
        println!("Original Vectors:\n\t{:?}\n\t{:?}", left, right);
        let originally_same = vector_same_set(&mut left, &mut right);
        let after_same = alternate_same_set(&mut left, &mut right);
        assert!(after_same == originally_same,
                "left: {:?}    right: {:?}\n\
                Fast version: {:?}, Slow Version: {:?}", left, right,
                originally_same, after_same);
    }

//     #[test]
//     fn set_check_permutation_independent() {
//         let initial_random = random_vector(MAX_VECTOR_SIZE);
//         let test_vector = rand_vec.clone();
//         for vector_permutation in /* Permutations of original vector */ {
//             assert!(vector_same_set(test_vector, vector_permutation));
//         }

//         let alt_rand_vec = random_vector(MAX_VECTOR_SIZE);
//         let is_same_set = vector_same_set(alt_rand_vec, test_vector);
//         for alt_vector_permutations in /* Permutations of new vector */ {
//             assert_eq!(vector_same_set(alt_vector_permutations, test_vector), is_same_set);
//         }
    // }

}
