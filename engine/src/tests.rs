use crate::engine::*;

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
		fruit_counts.insert(FruitType::Apple, 0);
		fruit_counts.insert(FruitType::Banana, 0);
		fruit_counts.insert(FruitType::Orange, 0);
		Player {
			position: pos,
			fruit_counts,
		}
	}

	#[test]
	fn move_valid() {
		let mut engine = blank_engine();
		engine.current_state.insert_player(blank_player(Position::new(4, 3)));
		engine.current_state.insert_player(blank_player(Position::new(2, 3)));
		engine.apply_actions((Action::Move(Direction::Down),
							  Action::Move(Direction::Right)));

		assert_eq!(engine.current_state.players.get(0).unwrap().position, Position::new(4, 4));
		assert_eq!(engine.current_state.players.get(1).unwrap().position, Position::new(3, 3));
	}

	#[test]
	fn move_oob() {
		let mut engine = blank_engine();
		engine.current_state.insert_player(blank_player(Position::new(0, 0)));
		engine.current_state.insert_player(blank_player(Position::new(4, 4)));
		engine.apply_actions((Action::Move(Direction::Up),
							  Action::Move(Direction::Right)));

		assert_eq!(engine.current_state.players.get(0).unwrap().position, Position::new(0, 0));
		assert_eq!(engine.current_state.players.get(1).unwrap().position, Position::new(4, 4));
	}

	#[test]
	fn move_same_place() {
		let mut engine = blank_engine();
		engine.current_state.insert_player(blank_player(Position::new(4, 3)));
		engine.current_state.insert_player(blank_player(Position::new(2, 3)));
		engine.apply_actions((Action::Move(Direction::Left),
							  Action::Move(Direction::Right)));

		assert_eq!(engine.current_state.players.get(0).unwrap().position, Position::new(4, 3));
		assert_eq!(engine.current_state.players.get(1).unwrap().position, Position::new(2, 3));
	}
}