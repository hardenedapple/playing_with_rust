/*
	Algorithm:
		Take a string type, have two pointers that iterate from either end.
		Each pointer stores elements into a String buffer one character at
		a time.
		When either pointer reaches a space, it moves its data into the
		other end of the total string.
		
	Note:
		we use [u8] instead of strings because we maintain the order of bytes
		in the string, and the only special character (space -- byte 32) never
		occurs in the middle of a unicode character.
		Hence working with 8 bit bytes instead of UTF-8 characters gives us the
		same answer while allowing us to index into a string efficiently.
		
	TODO:
		Currently assuming that the first character and the last character are
		not spaces.
		Check whether this would make a difference in the algorithm, and if so
		account for it later.
*/

use std::vec::Vec;
const ASCII_SPACE: u8 = 32;

/*
fn move_buffer_here(sentance: &mut [u8], buffer: &mut T, index: &usize)
	where T: Iterator<u8> {
	let mut count: usize = index;
	for character in buffer {
		sentance[count] = character;
		count += 1;
	}
}
*/

fn move_left_buffer(sentance: &mut [u8], left_buffer: &mut Vec<u8>, right_index: &usize) {
	let mut count: usize = right_index + 1;
	sentance[*right_index] = ASCII_SPACE;
	for character in left_buffer.drain(..) {
		sentance[count] = character;
		count += 1;
	}
}

fn move_right_buffer(sentance: &mut [u8], right_buffer: &mut Vec<u8>,
					 left_index: &usize, prev_left_index: &mut usize) {
	let mut count: usize = *prev_left_index;
	sentance[*left_index] = ASCII_SPACE;
	for character in right_buffer.drain(..).rev() {
		sentance[count] = character;
		count += 1;
	}
	*prev_left_index = *left_index + 1;
}


pub fn string_swap(sentance: &mut [u8]) {
	let mut left_word_buffer: Vec<u8> = Vec::new();
	let mut right_word_buffer: Vec<u8> = Vec::new();
	
	let (mut left_index, mut right_index,
		 mut prev_left_index): (usize, usize, usize) = (0, sentance.len() - 1, 0);
	loop {
		// Have reached the mid-point, break into cleanup code
		if left_index >= right_index {
			break;
		}
		let cur_right_char = sentance[right_index];
		let cur_left_char = sentance[left_index];

		if cur_left_char == ASCII_SPACE {
			move_left_buffer(sentance, &mut left_word_buffer, &right_index);
		} else {
			left_word_buffer.push(cur_left_char);
		}

		
		if cur_right_char == ASCII_SPACE {
			move_right_buffer(sentance, &mut right_word_buffer,
							  &left_index, &mut prev_left_index);
		} else {
			right_word_buffer.push(cur_right_char);
		}
	
		left_index += 1;
		right_index -= 1;
	}
	
	// In this cleanup code, we have one of the following situations:
	// two vectors with parts of words in them.
	// two vectors with entire words in them.
	// a single character not in either vector
	// no character not in either vector.
	if (left_index == right_index) && (sentance[left_index] == ASCII_SPACE) {
		move_left_buffer(sentance, &mut left_word_buffer, &right_index);
		move_right_buffer(sentance, &mut right_word_buffer,
						  &left_index, &mut prev_left_index);
		return;
	}

	if left_index == right_index { left_word_buffer.push(sentance[left_index]); }
	
	for character in left_word_buffer.drain(..) {
		sentance[prev_left_index] = character;
		prev_left_index += 1;
	}
	
	for character in right_word_buffer.drain(..).rev() {
		sentance[prev_left_index] = character;
		prev_left_index += 1;
	}
	
}

#[cfg(test)]
mod tests;