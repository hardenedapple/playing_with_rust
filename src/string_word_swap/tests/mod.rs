/*
	Test the string swapping algorithm
	Mainly by implementing the much simpler method, and testing that
	the results are the same.
	
	Have to watch out for double spaces -- the split() method gives you an empty string
	to represent them.
*/
use super::*;

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
}

#[test]
fn string_swap_compare_to_obvious() {
	/*
	let mut dictionary = ["hello", "there", "these", "are", "some", 
		"words", "that", "I",
		"can", "think", "of", "right", "now", "I'm", "not", "sure",
		"how", "I'm", "going", "to implement", "obtaining", "random",
		"words", "at", "the", "moment,", "but", "I", "expect", "it's",
		"something", "along", "the", "lines", "of making", "a",
		"random", "selection", "from", "two", "characters,", "space",
		"and", "'a',", "where", "with", "the", "selection", "of", "'a'
		being", "much", "more", "likely."].iter().map(| x | { String::from(x) });
	
	let mut total_words: Vec<String> = 
	let mut initial_sequence: Vec<u8> = random_sentance(MAX_LENGTH);
	*/
}
	