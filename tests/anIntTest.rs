extern crate rust_game_of_life;

#[cfg(test)]
mod tests {
	use rust_game_of_life::add_two;
	
	#[test]
	fn initial_board_correct_int() {
		assert_eq!(add_two(8), 10);
	}
	
}