/*
 * vimcmd: set makeprg=cargo\ test
 * vimcmd: !cargo test -- --ignored
 * vimcmd: !cargo test
 */

pub mod knapsack_problem;
pub mod string_word_swap;
pub mod disjoint_set;

#[cfg(test)]
pub mod test_utils;
