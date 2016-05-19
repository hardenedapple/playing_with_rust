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

/*
 * Because each new permutation is reached on the innermost recursive call, that call must have all
 * the information needed to make the callback.
 *
 * This means it must have a slice matching the entire vector passed in.
 *
 * Given this, in order for the recursive calls to know their depth, and how far in to the stack we
 * are, we have to pass them a position parameter.
 *
 * This is an implementation detail that I don't want to expose to the hypothetical user, so I
 * split the recursive part into a helper function and provide a wrapper as the entry point.
 */

fn heaps_algorithm_helper<F, T>(current: &mut [T], callback: &mut F, position: usize)
    where F: FnMut(&mut [T]) {
        if position == 1 {
            callback(current);
            return;
        } else if position == 0 {
            panic!("heaps_algorithm_helper() called with position == 0\n\
                    This should never happen -- investigate please.");
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
    let mut myvector = Vec::new();
    let mut complainer = | _: &mut [u32] | -> ! {
        panic!("Should not be called");
    };
    heaps_algorithm(&mut myvector, &mut complainer);
}

#[test]
#[should_panic(expected = "heaps_algorithm_helper() called with position == 0\n\
                    This should never happen -- investigate please.")]
fn recursive_call_panics_on_empty() {
    let mut rand_vec: Vec<u32> = random_vector(MAX_PERMUTATION_SIZE);
    heaps_algorithm_helper(&mut rand_vec, &mut | _: &mut [u32] | -> ! {
        panic!("Not correct panic");
    }, 0);
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

    /* Also test that the recursive part of the function immediately calls the array if "position"
     * is 1 no matter what the slice is. */
    let mut alternate_test: Vec<u32> = vec![1, 2, 3, 4];
    let samevector = alternate_test.clone();
    let mut altcallback = | test_array: &mut [u32] | {
        assert_eq!(test_array, &samevector as &[u32]);
    };
    heaps_algorithm_helper(&mut alternate_test, &mut altcallback, 1);
}

#[test]
#[ignore]
/// Use the recursive nature of the Heap's algorithm implementation to show correctness.
///
/// The reasoning is provided below:
///
/// We know from the tests above, that the function implements the base case properly (i.e. returns
/// a single element vector unmodified).
/// For each step on from that, we can take for granted that each recursive call of our function
/// using an array one smaller than the current one, correctly gives all permutations of an array
/// that size.
/// If we take that, we can just check that after each call to the recursive function we put an
/// element on the end of the array that hasn't been there before, and that each element in the
/// array appears at the end once only per recursive call, so overall (N-1)! times.
/// From implementation, it turns out that we're already calling the checking function on every new
/// permutation, so we throw in a few more checks that everything is in the correct order anyway.
fn creates_all_permutations() {
    let mut current_factorial = 1;

    let mut current_vector: Vec<usize> = vec![0];
    // Don't go higher than 10 -- factorial increases *really* fast.
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
/// Now we have reasonable confidence in our recursive implementation, we use it to check the
/// non-recursive implementation.
///
/// Just take a random vector, and iterate over all its permutations, asserting that both
/// implementations return the same permutations in the same order.
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
fn state_is_reset() {
    let permute_start: Vec<u32> = random_vector(MAX_PERMUTATION_SIZE);
    println!("Initial vector: {:?}", permute_start);
    let mut vector_permutations = VectorPermutations::from_vec(permute_start);
    let mut initial_state = vector_permutations.clone();
    while let Some(_) = vector_permutations.permute() {
    	// Just to iterate through the initial state.
    }
	
	assert_eq!(initial_state, vector_permutations);
	
	loop {
		// TODO -- extra block to satisfy the borrow checker
		//			any way to neaten this?
		{
			let initial = vector_permutations.permute();
			let alternate = initial_state.permute();
			
			if initial.is_none() {
				assert!(alternate.is_none());
				break;
			}
			assert_eq!(*(alternate.unwrap()), *(initial.unwrap()));
		}
		assert_eq!(initial_state, vector_permutations);
	}
	assert_eq!(initial_state, vector_permutations);
}

#[test]
#[ignore]
/// Testing random things is really difficult.
///
/// Here I just make sure we get the correct length output, completely ignoring the values in the
/// vector returned
fn random_vector_good_size() {
    // TODO -- make sure that we print this message out if someone kills the process.
    println!("Just to note -- this function could go on forever.\n\
              It's really unlikely to though, so I feel justified including it.\n\
              If it doesn't finish ^C will stop it, and you should find this message.");
    for i in 0..500 {
        let mut found_size = vec![false; i + 1];
        loop {
            // Could technically go on forever.
            // Is unlikely to do so
            let current_vector: Vec<u32> = random_vector(i);
            let cur_len = current_vector.len();
            assert!(cur_len <= i);
            found_size[cur_len] = true;
            if found_size == vec![true; i + 1] { break; };
        }
    }
}
