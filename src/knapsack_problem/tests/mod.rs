extern crate rand;
use knapsack_problem::*;
use test_utils::*;
pub const MAX_VECTOR_SIZE: usize = 30;
pub const MAX_PERMUTATION_SIZE: usize = 10;

fn alternate_same_set<T: PartialEq + Ord>(left: &mut Vec<T>, right: &mut Vec<T>) -> bool {
    left.sort(); right.sort();
    left == right
}

/*
 * NOTE:
 *  This function returns Items with attributes in the 16 bit range.
 *  This is so that with 30 elements in a knapsack, it should be impossible for the value or
 *  weight to overflow a 32 bit unsigned integer (unless I'm missing something.
 */
impl rand::Rand for Item {
    fn rand<R: rand::Rng>(rng: &mut R) -> Item {
        let tuple: (u16, u16) = rng.gen();
        Item { item_weight: tuple.0 as u32, item_value: tuple.1 as u32 }
    }
}

#[test]
fn handles_base_case() {
    let knapsack_problem = KnapsackProblem { kp_capacity: 0, kp_options: Vec::new() };
    let knapsack_solution = best_knapsack(knapsack_problem);
    let correct_solution = KnapsackSolution {
        ks_weight: 0, ks_value: 0, ks_capacity: 0, ks_items: Vec::new(),
    };
    assert_eq!(knapsack_solution, correct_solution);
}

#[test]
fn handles_simple() {
    let item_options = vec![
        Item { item_weight: 12, item_value: 4 },
        Item { item_weight: 1, item_value: 1 },
        Item { item_weight: 1, item_value: 2 },
        Item { item_weight: 2, item_value: 2 },
        Item { item_weight: 4, item_value: 10 }
    ];

    let knapsack_problem = KnapsackProblem {
        kp_capacity: 15,
        kp_options: item_options.clone(),
    };
    let correct_solution = KnapsackSolution {
        ks_weight: 8,
        ks_value: 15,
        ks_capacity: 7,
        ks_items: item_options[1..].to_vec(),
    };

    assert_eq!(best_knapsack(knapsack_problem), correct_solution);
}

#[test]
fn ignores_valueless() {
    let knapsack_problem = KnapsackProblem {
        kp_capacity: 10,
        kp_options: vec![ Item { item_weight: 1, item_value: 0 } ],
    };
    let knapsack_solution = best_knapsack(knapsack_problem);
    let correct_solution = KnapsackSolution {
        ks_weight: 0, ks_value: 0, ks_capacity: 10, ks_items: Vec::new(),
    };
    assert_eq!(knapsack_solution, correct_solution);
}

#[test]
fn same_result_each_time() {
    let rand_vec = random_vector(rand::random::<usize>() %
                                           MAX_VECTOR_SIZE);
    let random_capacity = rand::random::<u32>();

    let original_answer = best_knapsack(KnapsackProblem {
        kp_capacity: random_capacity,
        kp_options: rand_vec.clone(),
    });
    let other_answer = best_knapsack(KnapsackProblem {
        kp_capacity: random_capacity,
        kp_options: rand_vec.clone(),
    });

    assert_eq!(other_answer, original_answer);
}

#[test]
#[ignore]
fn order_insensitive() {
    /*
     * NOTE -- this once took over 15 min before I gave up.
     *		It took such a long time when using MAX_PERMUTATION_SIZE,
     *		so I have reduced it by one.
     *		I know that when reduced by one this will finish within
     *		that time limit because I attached to the running process in
     *		gdb and printed out the 'permutations' structure.
     *		It had reached the '9' level, but had only managed to get there
     *		4 times -- so waiting for level 10 was a little hopeless.
     */
    let item_options: Vec<Item> = random_vector(MAX_PERMUTATION_SIZE - 1);
    println!("Original vector: {:?}", item_options);
    let mut permutations = VectorPermutations::from_vec(item_options);
    let knapsack_capacity = rand::random();
    let first_solution: KnapsackSolution;
    if let Some(initial_permutation) = permutations.permute() {
        first_solution = best_knapsack(KnapsackProblem {
            kp_capacity: knapsack_capacity,
            kp_options: initial_permutation.clone(),
        });
    } else {
        return;
    }

    while let Some(next_permutation) = permutations.permute() {
        assert_eq!(first_solution, best_knapsack(KnapsackProblem {
            kp_capacity: knapsack_capacity,
            kp_options: next_permutation.clone(),
        }));
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
