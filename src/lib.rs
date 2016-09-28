/*
 * TODO Figure out how to enable experimental features.
 *      I want to only execute a line of code if I'm compiling for testing.

 * vimcmd: set makeprg=cargo\ test
 * vimcmd: !cargo test -- --ignored
 * vimcmd: !cargo test
 */

pub mod knapsack_problem;
pub mod string_word_swap;
pub mod disjoint_set;

#[cfg(test)]
pub mod test_utils;
