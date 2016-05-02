/*
 * TODO Documentation tests -- see here
 *      https://doc.rust-lang.org/book/testing.html#documentation-tests
 *
 *      Testing the tests
 *          Not sure about testing that random_vector() returns an actually random vector
 *              Seems like a lot of trouble to get too few assertions.
 *
 *          Testing the VectorPermutations stuff should be reasonably straightforward for small
 *          vectors, but I'd like to push the test as far forward as possible.
 *              To start with, (just to add some confidence -- it doesn't really proove anything)
 *              is to iterate through all permutations asserting that they aren't equal to the
 *              original one.
 *
 *              After that we'll need something more complex for more assurances.
 *
 *              First guess:
 *                  Test that there are no duplicates in any permutation of [1, 2, 3, 4]
 *                  Ensure that each element is in each position (N-1)! times
 *                      Array counting the number of times each element has been in a position.
 *
 *              Alternative test:
 *                  Ensure that all permutations are reached for N == 3.
 *                  For each N onwards (up to 10), ensure that each element is put in the last
 *                  position N-1 times.
 *                  Doesn't actually proove the algorithm has been implemented correctly, but
 *                  assuming the subset of the array reaches each permutation correctly it works.
 *
 *              Third alternative:
 *                  Functionally implement the recursive version of Heap's algorithm.
 *                  Test this recursive version, using the induction hypothesis (which we can
 *                  proove applies because we're reusing the same code without holding state).
 *                  Compare the results between each version.
 *
 * vimcmd: set makeprg=cargo\ test
 * vimcmd: !cargo test -- --ignored
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
    const MAX_PERMUTATION_SIZE: usize = 10;

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
     *  TODO -- actually check that this is the case, so far this conclusion is only justified by
     *  inspection.
     */
    impl rand::Rand for Item {
        fn rand<R: rand::Rng>(rng: &mut R) -> Item {
            let tuple: (u16, u16) = rng.gen();
            Item { weight: tuple.0 as u32, value: tuple.1 as u32 }
        }
    }

    /*
     * TODO
     * If I make the distribution less random and more weighted to the edges, this may catch more
     * edge cases in the code.
     * See https://doc.rust-lang.org/rand/rand/distributions/gamma/index.html
     * https://doc.rust-lang.org/rand/rand/distributions/index.html
     * for how to select alternate distributions.
     */
    fn random_vector<T: rand::Rand>(max_length: usize)
        -> Vec<T> {
            if max_length == 0 {
                return Vec::new();
            }

            let length: usize = rand::random::<usize>() % (max_length + 1);
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

    struct VectorPermutations<T> {
        permutation: Vec<T>,
        count: Vec<usize>,
        started: bool,
        index: usize,
    }

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

        fn permute<'a>(&'a mut self) -> Option<&'a Vec<T>> {
            // Start and end conditions
            let num_elements = self.permutation.len();
            if !self.started {
                if self.permutation.is_empty() {
                    return None;
                }
                self.started = true;
                // Return original permutation as first one.
                return Some(&self.permutation);
            } else if self.index == num_elements {
                // This is our finished condition
                return None;
            }

            // Move one more step along Heap's algorithm
            while self.index < num_elements {
                if self.count[self.index] < self.index {
                    let swap_index = self.index % 2 * self.count[self.index];
                    self.permutation.swap(swap_index, self.index);
                    self.count[self.index] += 1;
                    self.index = 1;
                    return Some(&self.permutation);
                } else {
                    self.count[self.index] = 0;
                    self.index += 1;
                }
            }
            None
        }
    }

    #[test]
    #[ignore]
    fn order_insensitive() {
        let item_options: Vec<Item> = random_vector(MAX_PERMUTATION_SIZE);
        println!("Original vector: {:?}", item_options);
        let mut permutations = VectorPermutations::from_vec(item_options);
        let knapsack_capacity = rand::random();
        let first_solution: KnapsackSolution;
        if let Some(initial_permutation) = permutations.permute() {
            first_solution = best_knapsack(KnapsackProblem {
                capacity: knapsack_capacity,
                options: initial_permutation.clone(),
            }).unwrap();
        } else {
            return;
        }

        while let Some(next_permutation) = permutations.permute() {
            assert_eq!(first_solution, best_knapsack(KnapsackProblem {
                capacity: knapsack_capacity,
                options: next_permutation.clone(),
            }).unwrap());
        }
    }

    fn alternate_same_set<T: PartialEq + Ord>(left: &mut Vec<T>, right: &mut Vec<T>) -> bool {
        left.sort(); right.sort();
        left == right
    }

    #[test]
    fn set_comparison() {
        let mut left: Vec<u32> = random_vector(MAX_VECTOR_SIZE);
        let mut right: Vec<u32>;
        if rand::random::<bool>() {
            right = left.clone();
        } else {
            right = random_vector(MAX_VECTOR_SIZE);
        }
        println!("Original Vectors:\n\t{:?}\n\t{:?}", left, right);
        let originally_same = vector_same_set_test(&mut left, &mut right);
        let after_same = alternate_same_set(&mut left, &mut right);
        assert!(after_same == originally_same,
                "left: {:?}    right: {:?}\n\
                Fast version: {:?}, Slow Version: {:?}", left, right,
                originally_same, after_same);
    }

    #[test]
    #[ignore]
    fn set_check_permutation_independent() {
        let initial_random: Vec<u32> = random_vector(MAX_PERMUTATION_SIZE);
        println!("Initial vector: {:?}", initial_random);
        let test_vector = initial_random.clone();
        let mut vector_permutations = VectorPermutations::from_vec(initial_random);
        while let Some(new_permutation) = vector_permutations.permute() {
            assert!(vector_same_set_test(&test_vector, &new_permutation));
        }

        let alt_rand_vec: Vec<u32> = random_vector(MAX_PERMUTATION_SIZE);
        println!("Alternate vector: {:?}", alt_rand_vec);
        let is_same_set = vector_same_set_test(&alt_rand_vec, &test_vector);
        vector_permutations = VectorPermutations::from_vec(alt_rand_vec);
        while let Some(new_permutation) = vector_permutations.permute() {
            assert_eq!(vector_same_set_test(new_permutation, &test_vector), is_same_set);
        }
    }

}
