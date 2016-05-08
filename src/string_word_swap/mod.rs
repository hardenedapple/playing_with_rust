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
*/

use std::vec::Vec;
const ASCII_SPACE: u8 = 32;

fn move_buffer_here<T>(sentance: &mut [u8], buffer: T, index: usize)
	where T: Iterator<Item=u8> {
	let mut count: usize = index;
	for character in buffer {
		sentance[count] = character;
		count += 1;
	}
}


pub fn string_swap(sentance: &mut [u8]) {
	let mut left_word_buffer: Vec<u8> = Vec::new();
	let mut right_word_buffer: Vec<u8> = Vec::new();
	
	if sentance.len() == 0 {
		return;
	}
	
	let (mut left_index, mut right_index,
		 mut prev_left_index): (usize, usize, usize) = (0, sentance.len() - 1, 0);

	macro_rules! move_left_buffer {
		() => {
				sentance[right_index] = ASCII_SPACE;
				move_buffer_here(sentance, left_word_buffer.drain(..), right_index + 1);
		};
	}

	macro_rules! move_right_buffer {
		() => {
				sentance[left_index] = ASCII_SPACE;
				move_buffer_here(sentance, right_word_buffer.drain(..).rev(), prev_left_index);
				prev_left_index = left_index + 1;
		};
	}

	loop {
		// Have reached the mid-point, break into cleanup code
		if left_index >= right_index {
			break;
		}
		let cur_right_char = sentance[right_index];
		let cur_left_char = sentance[left_index];

		if cur_left_char == ASCII_SPACE {
			move_left_buffer!();
		} else {
			left_word_buffer.push(cur_left_char);
		}

		
		if cur_right_char == ASCII_SPACE {
			move_right_buffer!();
		} else {
			right_word_buffer.push(cur_right_char);
		}
	
		left_index += 1;
		right_index -= 1;
	}
	
	if (left_index == right_index) && (sentance[left_index] == ASCII_SPACE) {
		move_left_buffer!();
		move_right_buffer!();
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