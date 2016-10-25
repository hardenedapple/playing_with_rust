/*
	Test the string swapping algorithm
	Mainly by implementing the much simpler method, and testing that
	the results are the same.

	Have to watch out for double spaces -- the split() method gives you an empty string
	to represent them.
*/
use super::*;
use test_utils::{random_vector, seeded_rng};
use test_utils::rand::Rng;


#[test]
fn string_swap_works_basic() {
	let mut initial_sentance: Vec<u8> = String::from("Hello there this \
														is an example").into_bytes();
	let correct_answer: Vec<u8> = String::from("example an is \
												this there Hello").into_bytes();
	string_swap(&mut initial_sentance);
	assert_eq!(initial_sentance, correct_answer);
}

#[test]
fn inplace_string_swap_works_basic() {
	let mut initial_sentance: Vec<u8> = String::from("Hello there this \
														is an example").into_bytes();
	let correct_answer: Vec<u8> = String::from("example an is \
												this there Hello").into_bytes();
	inplace_string_swap(&mut initial_sentance);
	assert_eq!(initial_sentance, correct_answer);
}

fn obvious_word_reversal(sentance: &str) -> String {
	sentance.split(' ').rev().collect::<Vec<&str>>().join(" ")
}

#[test]
fn obvious_version_works() {
	let answer = obvious_word_reversal("Hello there this is an example");
	let correct_answer = String::from("example an is this there Hello");
	assert_eq!(answer, correct_answer);
}

fn all_versions_agree(original_sentance: &str) {
	let mut raw_vector: Vec<u8> = String::from(original_sentance).into_bytes();
	string_swap(&mut raw_vector);
	let word_buffered: String = unsafe { String::from_utf8_unchecked(raw_vector) };

	let simple_version = obvious_word_reversal(original_sentance);
	assert_eq!(word_buffered, simple_version); // Check for requiring borrow here

	let mut raw_vector: Vec<u8> = String::from(original_sentance).into_bytes();
	inplace_string_swap(&mut raw_vector);
	let inplace_version: String = unsafe { String::from_utf8_unchecked(raw_vector) };

	assert_eq!(inplace_version, simple_version);
}

#[test]
fn known_edge_cases() {
	all_versions_agree(" this sentance has a space at the start");
	all_versions_agree("this one has a space at the end ");
	all_versions_agree("sentance witha space as the middle char");
	all_versions_agree("sentance with many                      spaces in the middle");
	all_versions_agree("the middle of this sentance is a part of an word");
	all_versions_agree("   this sentance  has multiple   spaces    in different  places  ");
	all_versions_agree("onelongwordthatshouldn'tchange");
	all_versions_agree("similarwithoutanmiddlecharacter");
	all_versions_agree("This sentance has no middlle char and the middle is in a word");
	all_versions_agree("similarly, the break of this sen tance is at the start of a word");
	all_versions_agree("similarly the break of this sen tance is at the start of a word");
	all_versions_agree("while this sentance is sp lit at the end of a word");
	all_versions_agree("while this sentance is sp lit at the end of an word");
	all_versions_agree("");
	all_versions_agree(" ");
	all_versions_agree("t ");
	all_versions_agree(" t");
	all_versions_agree("  ");
	all_versions_agree(" a o e u i ' q j k x , . - p y b m w v z d h t n s f g c r l ");
}

fn random_space_positions(max_size: usize, max_position: usize) -> Vec<usize> {
    // No point having many numbers greater than the length of the string.
    // Some numbers greater than that length means we can have spaces on the end.
    let mut rand_vec = random_vector(max_size).iter().map(
        |x| x % max_position).collect::<Vec<usize>>();
	rand_vec.sort();
    rand_vec.dedup();

    // force ~ 50% chance of double spaces (except when rand_vec == 0)
    let mut rng = seeded_rng();
    if rng.gen::<bool>() && rand_vec.len() > 0 {
        let num_duplicates = rng.gen::<usize>() % rand_vec.len();
        for _ in 0..num_duplicates {
            let dup_position = rng.gen::<usize>() % rand_vec.len();
            let dup_val = rand_vec[dup_position];
            rand_vec.insert(dup_position, dup_val);
        }
    }

    rand_vec
}

fn create_random_sentance(max_size: usize)
	 -> String {
	let mut initial_sentance = String::from(
										"abcdefghijklmnopqrstuvwxyz\
										0123456789\
										ABCDEFGHIJKLMNOPQRSTUVWXYZ");

	let mut final_string = String::new();
	let mut current_position: usize = 0;
    for next_space in random_space_positions(max_size, initial_sentance.len() + 2) {
		for _ in current_position..next_space {
			if let Some(next_char) = initial_sentance.pop() {
				final_string.push(next_char);
			} else {
				break;
			}
		}
		current_position = next_space;
		final_string.push(' ');
	}

	for char in initial_sentance.chars().rev() {
		final_string.push(char);
	}

	final_string
}

#[test]
fn all_agree_random() {
	for _ in 0..1000 {
		let next_sentance = create_random_sentance(20);
		all_versions_agree(&next_sentance);
	}
}

