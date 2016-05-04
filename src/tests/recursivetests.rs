/*
 * This module is here to test the functions defined in the tests module.
 * (hence the module name).
 *
 * Currently the only things I'm testing are that the non-recursive implementation of Heap's
 * algorithm gives the same answer as the recursive implementation given here.
 *
 * We also test the recursive implementation, because we can use the fact that it's recursive to
 * aid us when asserting statements about the correctness.
 */
use super::*;

fn heaps_algorithm_helper<F, T>(current: &mut [T], callback: &mut F, position: usize)
    where F: FnMut(&mut [T]) {
        if position == 1 {
            callback(current);
            return;
        } else if position == 0 {
            panic!("I don't think this should ever happen");
        }

        for i in 0..(position - 1) {
            heaps_algorithm_helper(current, callback, position - 1);
            let swap_index = if position % 2 == 0 { i } else { 0 };
            current.swap(swap_index, position - 1);
        }

        heaps_algorithm_helper(current, callback, position - 1);
}

fn heaps_algorithm<F, T>(mut current: &mut Vec<T>, callback: &mut F)
    where F: FnMut(&mut [T]) {
        let array_len = current.len();
        if array_len <= 1 {
            if array_len == 1 { callback(&mut current); };
            return;
        }

        heaps_algorithm_helper(&mut current, callback, array_len);

        /* Return the vector we were given back to its original order.
         * This is based on Heap's algorithm invariants in the proof.
         * When the length of the array is even, the resulting permutation from the original
         * [a0, a1, a2, ..., a(n-1)]
         *
         * is
         * [a(n-3), a(n-2), 1, 2, ..., a(n-4), a(n-1), a0]
         *
         * while when the length of the array is odd, the resulting permutation from the original
         * is
         * [a(n-1), 1, 2, ..., a(n-2), a0]
         */
        if array_len % 2 == 0 {
            current.swap(array_len - 1, 1);
            current.swap(array_len - 2, 0);
            let last_element = current.remove(0);
            current.push(last_element);
        } else {
            current.swap(array_len - 1, 0);
        }
}

#[test]
fn permuter_handles_empty() {
    let mut myvector: Vec<u32> = vec![];
    heaps_algorithm(&mut myvector, &mut | _: &mut [u32] | -> ! { panic!("Should not be called"); });
}

#[test]
fn permuter_handles_base_case() {
    let mut testvector: Vec<u32> = vec![1];
    let samevector = testvector.clone();
    let mut callback = | test_array: &mut [u32] | {
        assert_eq!(test_array, &samevector as &[u32]);
    };
    heaps_algorithm(&mut testvector, &mut callback);
    heaps_algorithm_helper(&mut testvector, &mut callback, 1);
}

/*
 * This function uses the recursive nature of the Heap's algorithm implementation to show the
 * correctness of its output.
 *
 * The reasoning is provided below:
 *      We know from the tests above, that the function implements the base case properly (i.e.
 *      returns a single element vector unmodified)
 *      For each step on from that, we can take for granted that each recursive call of our
 *      function using an array one smaller than the current one, correctly gives all permutations
 *      of an array that size.
 *      If we take that, we can just check that after each call to the recursive function we put an
 *      element on the end of the array that hasn't been there before, and that each element in the
 *      array appears at the end once only (per recursive call, so overall (N-1)! times.
 *      From implementation details, it turns out that we're already calling the checking function
 *      on every new permutation, so we throw in a few more checks that everything is in the
 *      correct order anyway.
 */
#[test]
#[ignore]
fn creates_all_permutations() {
    let mut current_factorial = 1;

    let mut current_vector: Vec<usize> = vec![0];
    for last_element in 1..11 {
        let mut times_seen = vec![0; last_element + 1];
        let mut prev_last = last_element;
        let mut count = 0;
        current_factorial = last_element * current_factorial;
        current_vector.push(last_element);

        { // Just create a scope so the mutable borrow of "times_seen" is known to finish before we
          // immutably borrow it to check it turned out as it should.
            let mut array_checker = | array_check: &mut [usize] | {
                let current_last = array_check.last().unwrap();
                if count < current_factorial {
                    assert_eq!(*current_last, prev_last);
                    count += 1;
                } else {
                    assert!(times_seen[*current_last] == 0);
                    assert!(times_seen[prev_last] == current_factorial);
                    prev_last = *current_last;
                    count = 1;
                }
                times_seen[*current_last] += 1;
            };
            heaps_algorithm(&mut current_vector, &mut array_checker);
        }
        assert!(times_seen.iter().all(| item: &usize | -> bool { *item == current_factorial }));
    }
}

#[test]
fn same_permutations() {
    let initial_random: Vec<u32> = random_vector(MAX_PERMUTATION_SIZE);
    println!("Initial vector: {:?}", initial_random);
    let mut test_vector = initial_random.clone();
    let mut vector_permutations = VectorPermutations::from_vec(initial_random);
    let mut count = 0;
    {
        let mut callback = | array_check: &mut [u32] | {
            count += 1;
            if let Some(new_permutation) = vector_permutations.permute() {
                assert_eq!(array_check, new_permutation as &[u32]);
            } else {
                panic!("Element number {:?} of the VectorPermutations structure missing.",
                    count);
            }
        };
        heaps_algorithm(&mut test_vector, &mut callback);
    }
    assert!(vector_permutations.permute().is_none());
}

#[test]
fn random_vector_good_size() {
    for i in 0..20 {
        for _ in 0..2000 {
            assert!((random_vector(i) as Vec<u32>).len() <= i);
        }
    }
}
