use crate::engine::*;

#[allow(unused_imports)]
#[cfg(test)]
mod tests {

	use crate::engine::*;
	use std::collections::HashMap;

	fn blank_engine() -> Engine {
		Engine::new(EngineConfig {
			board_size: 5,
			populate_board: false,
			fruit_density: 0.0,
		})
	}

	fn blank_player(pos: Position) -> Player {
		let mut fruit_counts = HashMap::new();
		fruit_counts.insert(FruitType::Apple, 0.0f32);
		fruit_counts.insert(FruitType::Banana, 0.0f32);
		fruit_counts.insert(FruitType::Orange, 0.0f32);
		Player {
			position: pos,
			fruit_counts,
		}
	}

	#[test]
	fn move_valid() {
		let mut engine = blank_engine();
		engine.current_state.player1 = blank_player(Position::new(4, 3));
		engine.current_state.player2 = blank_player(Position::new(2, 3));
		engine.apply_actions((Action::Move(Direction::Down),
							  Action::Move(Direction::Right)));

		assert_eq!(engine.current_state.player1.position, Position::new(4, 4));
		assert_eq!(engine.current_state.player2.position, Position::new(3, 3));
	}

	#[test]
	fn move_oob_valid_moves() {
		let mut engine = blank_engine();
		engine.current_state.player1 = blank_player(Position::new(0, 0));
		engine.current_state.player2 = blank_player(Position::new(4, 4));

		let p1_valid_moves = engine.get_valid_moves(&engine.current_state.player1);
		assert!(!p1_valid_moves.contains(&Action::Move(Direction::Up)));
		assert!(p1_valid_moves.contains(&Action::Move(Direction::Down)));

		engine.apply_actions((Action::Move(Direction::Up),
							  Action::Move(Direction::Right)));

		assert_eq!(engine.current_state.player1.position, Position::new(0, 0));
		assert_eq!(engine.current_state.player2.position, Position::new(4, 4));
	}

	#[test]
	fn move_same_pos() {
		let mut engine = blank_engine();
		engine.current_state.player1 = blank_player(Position::new(4, 3));
		engine.current_state.player2 = blank_player(Position::new(2, 3));
		engine.apply_actions((Action::Move(Direction::Left),
							  Action::Move(Direction::Right)));

		assert_eq!(engine.current_state.player1.position, Position::new(4, 3));
		assert_eq!(engine.current_state.player2.position, Position::new(2, 3));
	}

	#[test]
	fn move_pickup_fruit() {
		let mut engine = blank_engine();
		engine.current_state.player1 = blank_player(Position::new(4, 3));
		engine.current_state.player2 = blank_player(Position::new(2, 3));

		engine.current_state.board.set_fruit(3, 3, Some(FruitType::Orange));
		assert_eq!(engine.current_state.board.get_fruit(3, 3), Some(FruitType::Orange));
		assert_eq!(*engine.current_state.player1.get_fruit_count(FruitType::Orange), 0.0f32);

		engine.apply_actions((Action::Move(Direction::Left),
							  Action::DoNothing));
		assert_eq!(engine.current_state.board.get_fruit(3, 3), None);
		assert_eq!(*engine.current_state.player1.get_fruit_count(FruitType::Orange), 1.0f32);
	}

	#[test]
	fn move_pickup_fruit_same_pos() {
		let mut engine = blank_engine();
		engine.current_state.player1 = blank_player(Position::new(4, 3));
		engine.current_state.player2 = blank_player(Position::new(2, 3));

		engine.current_state.board.set_fruit(3, 3, Some(FruitType::Orange));
		assert_eq!(engine.current_state.board.get_fruit(3, 3), Some(FruitType::Orange));
		assert_eq!(*engine.current_state.player1.get_fruit_count(FruitType::Orange), 0.0f32);

		engine.apply_actions((Action::Move(Direction::Left),
							  Action::Move(Direction::Right)));
		assert_eq!(engine.current_state.board.get_fruit(3, 3), None);
		assert_eq!(*engine.current_state.player1.get_fruit_count(FruitType::Orange), 0.5f32);
		assert_eq!(*engine.current_state.player2.get_fruit_count(FruitType::Orange), 0.5f32);

		assert_eq!(*engine.current_state.player2.get_fruit_count(FruitType::Apple), 0.0f32);
	}
}