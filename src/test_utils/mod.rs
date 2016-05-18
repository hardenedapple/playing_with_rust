extern crate rand;
use self::rand::Rng;

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
