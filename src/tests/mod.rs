extern crate rand;
use super::*;
use self::rand::Rng;
pub const MAX_VECTOR_SIZE: usize = 30;
pub const MAX_PERMUTATION_SIZE: usize = 10;

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
pub fn random_vector<T: rand::Rand>(max_length: usize)
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

pub struct VectorPermutations<T> {
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

fn alternate_same_set<T: PartialEq + Ord>(left: &mut Vec<T>, right: &mut Vec<T>) -> bool {
    left.sort(); right.sort();
    left == right
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

#[test]
#[ignore]
fn order_insensitive() {
    /*
     * NOTE -- this once took over 15 min before I gave up -- figure out the maximum running time
     * of this function and get it under control.
     */
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

mod recursivetests;
