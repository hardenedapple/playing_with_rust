// Publically declare the rand crate here because I (so far) only use it in test modules when I'm
// already 'use'ing from this module anyway.
pub extern crate rand;
use self::rand::Rng;
/*
	Recommended maximum sizes for vectors with permutations.
	We export this because iterating over all permutations in a vector
	is factorial in the number of elements, so more than about 10
	becomes prohibitively expensive.
	This is something that users of this code should know and have
	available to them.
*/

pub const MAX_PERMUTATION_SIZE: usize = 10;

pub fn seeded_rng() -> rand::StdRng {
    /*
     * Running random tests has the benefit that you're more likely to get test cases you haven't
     * thought of, but the downside of reproducability.
     * In order to aid in reproduction I like to randomly generate a seed, and print it out so that
     * if something goes wrong I can manually use that seed when debugging it.
     * Usually in C I pass this seed in as a command line argument, but I don't know how to do that
     * through Cargo, so I make sure all seeds come from this function so that I am able to
     * hard-code the seed in here if required.
     */
    let seed = rand::random::<[usize; 4]>();
    println!("seed: {:?}", seed);
    rand::SeedableRng::from_seed(&seed as &[usize])
}

/*
 * TODO (maybe)
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

        let mut rng = seeded_rng();
        let length: usize = rng.gen::<usize>() % (max_length + 1);
        let mut retval: Vec<T> = Vec::with_capacity(length);
        for _ in 0..length {
            retval.push(rng.gen());
        }
        retval
}

#[derive(Clone, PartialEq, Debug)]
pub struct VectorPermutations<T> {
    permutation: Vec<T>,
    count: Vec<usize>,
    started: bool,
    index: usize,
}

impl <T> VectorPermutations<T> {
    pub fn from_vec(original_vector: Vec<T>)
        -> VectorPermutations<T> {
            let len = original_vector.len();
            VectorPermutations {
                permutation: original_vector,
                count: vec![0; len],
                started: false,
                index: 0,
            }
    }

    pub fn permute<'a>(&'a mut self) -> Option<&'a Vec<T>> {
        // Start and end conditions
        let num_elements = self.permutation.len();
        if !self.started {
            if self.permutation.is_empty() {
                return None;
            }
            self.started = true;
            // Return original permutation as first one.
            return Some(&self.permutation);
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

        /*
        	We have finished, and are about to return None.
        	To allow repeated use of this structure, we reset the value
        	to its initialised state, so that calling this function
        	again will start from the beginning.
        */
        if num_elements % 2 == 0 {
            self.permutation.swap(num_elements - 1, 1);
            self.permutation.swap(num_elements - 2, 0);
            let last_element = self.permutation.remove(0);
            self.permutation.push(last_element);
        } else {
            self.permutation.swap(num_elements - 1, 0);
        }

        for value in self.count.iter_mut() {
        	*value = 0;
        }

        self.started = false;
        self.index = 0;
        None
    }
}

mod recursivetests;
