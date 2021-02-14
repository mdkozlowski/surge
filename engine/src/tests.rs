#[allow(unused_imports)]
#[cfg(test)]
mod tests {
	use crate::engine::*;
	use std::collections::HashMap;
	use crate::engine::FruitType::*;
	use crate::engine::Direction::*;

	fn blank_engine() -> Engine {
		Engine::new(EngineConfig {
			board_size: 5,
			populate_board: false,
			fruit_density: 0.0,
			random_seed: 123
		})
	}

	#[test]
	fn fruit_counts_update() {
		let mut engine = blank_engine();

		assert_eq!(*engine.current_state.board.fruit_counts.get(&Orange).unwrap(), 0);
		engine.current_state.board.set_fruit(3, 3, Some(Orange));
		assert_eq!(*engine.current_state.board.fruit_counts.get(&Orange).unwrap(), 1);
	}

	#[test]
	fn move_valid() {
		let mut engine = blank_engine();
		engine.current_state.player1 = Player::new(Position::new(4, 3));
		engine.current_state.player2 = Player::new(Position::new(2, 3));
		engine.apply_move((Action::Move(Direction::Down),
						   Action::Move(Direction::Right)));

		assert_eq!(engine.current_state.player1.position, Position::new(4, 4));
		assert_eq!(engine.current_state.player2.position, Position::new(3, 3));
	}

	#[test]
	fn move_oob_valid_moves() {
		let mut engine = blank_engine();
		engine.current_state.player1 = Player::new(Position::new(0, 0));
		engine.current_state.player2 = Player::new(Position::new(4, 4));

		let p1_valid_moves = engine.get_valid_moves(&engine.current_state.player1);
		assert!(!p1_valid_moves.contains(&Action::Move(Direction::Up)));
		assert!(p1_valid_moves.contains(&Action::Move(Direction::Down)));

		engine.apply_move((Action::Move(Direction::Up),
						   Action::Move(Direction::Right)));

		assert_eq!(engine.current_state.player1.position, Position::new(0, 0));
		assert_eq!(engine.current_state.player2.position, Position::new(4, 4));
	}

	#[test]
	fn move_same_pos() {
		let mut engine = blank_engine();
		engine.current_state.player1 = Player::new(Position::new(4, 3));
		engine.current_state.player2 = Player::new(Position::new(2, 3));
		engine.apply_move((Action::Move(Direction::Left),
						   Action::Move(Direction::Right)));

		assert_eq!(engine.current_state.player1.position, Position::new(4, 3));
		assert_eq!(engine.current_state.player2.position, Position::new(2, 3));
	}

	#[test]
	fn move_pickup_fruit() {
		let mut engine = blank_engine();
		engine.current_state.player1 = Player::new(Position::new(4, 3));
		engine.current_state.player2 = Player::new(Position::new(2, 3));

		engine.current_state.board.set_fruit(3, 3, Some(Orange));
		assert_eq!(engine.current_state.board.get_fruit(3, 3), Some(Orange));
		assert_eq!(*engine.current_state.player1.get_fruit_count(Orange), 0.0f32);

		engine.apply_move((Action::Move(Direction::Left),
						   Action::DoNothing));
		assert_eq!(engine.current_state.board.get_fruit(3, 3), None);
		assert_eq!(*engine.current_state.player1.get_fruit_count(Orange), 1.0f32);
		assert_eq!(*engine.current_state.board.fruit_counts.get(&Orange).unwrap(), 0);
	}

	#[test]
	fn move_pickup_fruit_same_pos() {
		let mut engine = blank_engine();
		engine.current_state.player1 = Player::new(Position::new(4, 3));
		engine.current_state.player2 = Player::new(Position::new(2, 3));

		engine.current_state.board.set_fruit(3, 3, Some(Orange));
		assert_eq!(engine.current_state.board.get_fruit(3, 3), Some(Orange));
		assert_eq!(*engine.current_state.player1.get_fruit_count(Orange), 0.0f32);

		engine.apply_move((Action::Move(Direction::Left),
						   Action::Move(Direction::Right)));
		assert_eq!(engine.current_state.board.get_fruit(3, 3), None);
		assert_eq!(*engine.current_state.player1.get_fruit_count(Orange), 0.5f32);
		assert_eq!(*engine.current_state.player2.get_fruit_count(Orange), 0.5f32);

		assert_eq!(*engine.current_state.player2.get_fruit_count(Apple), 0.0f32);
	}

	#[test]
	fn win_condition_p1_winner() {
		let mut engine = blank_engine();
		engine.current_state.board.set_fruit(2, 2, Some(Apple));
		engine.current_state.board.set_fruit(2, 3, Some(Banana));
		engine.current_state.board.set_fruit(2, 4, Some(Orange));

		engine.current_state.player1 = Player::new(Position::new(2, 1));
		engine.current_state.player2 = Player::new(Position::new(4, 4));

		// engine.current_state.print_state();

		let win_state = engine.apply_move((Action::Move(Direction::Down), Action::DoNothing));
		assert_eq!(win_state, WinState::InProgress);

		let win_state = engine.apply_move((Action::Move(Direction::Down), Action::DoNothing));
		assert_eq!(win_state, WinState::InProgress);

		let win_state = engine.apply_move((Action::Move(Direction::Down), Action::DoNothing));
		assert_eq!(win_state, WinState::Finished(PlayerWinner::Player1));
	}

	#[test]
	fn win_condition_p2_winner() {
		let mut engine = blank_engine();
		engine.current_state.board.set_fruit(2, 2, Some(Apple));
		engine.current_state.board.set_fruit(2, 3, Some(Banana));
		engine.current_state.board.set_fruit(2, 4, Some(Orange));

		engine.current_state.player1 = Player::new(Position::new(4, 4));
		engine.current_state.player2 = Player::new(Position::new(2, 1));

		// engine.current_state.print_state();

		let win_state = engine.apply_move((Action::DoNothing, Action::Move(Direction::Down)));
		assert_eq!(win_state, WinState::InProgress);

		let win_state = engine.apply_move((Action::DoNothing, Action::Move(Direction::Down)));
		assert_eq!(win_state, WinState::InProgress);

		let win_state = engine.apply_move((Action::DoNothing, Action::Move(Direction::Down)));
		assert_eq!(win_state, WinState::Finished(PlayerWinner::Player2));
	}

	#[test]
	fn fruit_counts_p1_winner() {
		let mut engine = blank_engine();
		engine.current_state.board.update_fruit_counts();
		engine.current_state.board.set_fruit(2,2, Some(Apple));

		engine.current_state.player1 = Player::new(Position::new(4, 4));
		engine.current_state.player2 = Player::new(Position::new(2, 1));

		let win_state = engine.apply_move((Action::DoNothing, Action::DoNothing));
		assert_eq!(win_state, WinState::InProgress);

		*engine.current_state.player1.fruit_counts.get_mut(&Apple).unwrap() = 7.0f32;
		*engine.current_state.player1.fruit_counts.get_mut(&Banana).unwrap() = 5.0f32;
		*engine.current_state.player1.fruit_counts.get_mut(&Orange).unwrap() = 2.0f32;

		*engine.current_state.player2.fruit_counts.get_mut(&Apple).unwrap() = 3.0f32;
		*engine.current_state.player2.fruit_counts.get_mut(&Banana).unwrap() = 3.0f32;
		*engine.current_state.player2.fruit_counts.get_mut(&Orange).unwrap() = 20.0f32;

		let win_state = engine.apply_move((Action::DoNothing, Action::Move(Down)));
		assert_eq!(win_state, WinState::Finished(PlayerWinner::Player1));
	}

	#[test]
	fn fruit_counts_p2_winner() {
		let mut engine = blank_engine();
		engine.current_state.board.update_fruit_counts();
		engine.current_state.board.set_fruit(2,2, Some(Apple));

		engine.current_state.player1 = Player::new(Position::new(4, 4));
		engine.current_state.player2 = Player::new(Position::new(2, 1));

		let win_state = engine.apply_move((Action::DoNothing, Action::DoNothing));
		assert_eq!(win_state, WinState::InProgress);

		*engine.current_state.player1.fruit_counts.get_mut(&Apple).unwrap() = 5.0f32;
		*engine.current_state.player1.fruit_counts.get_mut(&Banana).unwrap() = 5.0f32;
		*engine.current_state.player1.fruit_counts.get_mut(&Orange).unwrap() = 1.0f32;

		*engine.current_state.player2.fruit_counts.get_mut(&Apple).unwrap() = 2.0f32;
		*engine.current_state.player2.fruit_counts.get_mut(&Banana).unwrap() = 6.0f32;
		*engine.current_state.player2.fruit_counts.get_mut(&Orange).unwrap() = 10.0f32;

		let win_state = engine.apply_move((Action::DoNothing, Action::Move(Down)));
		assert_eq!(win_state, WinState::Finished(PlayerWinner::Player2));
	}
}