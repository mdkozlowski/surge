#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use engine::engine::{Engine, EngineConfig, FruitType, Direction, Action};
	use rand::{thread_rng, seq};
	use rand::seq::IteratorRandom;

	fn blank_engine() -> Engine {
		Engine::new(EngineConfig {
			board_size: 5,
			populate_board: true,
			fruit_density: 0.3f32,
			random_seed: 123,
		})
	}

	fn fruit_counts_update() {
		let mut rng = thread_rng();
		let mut engine = blank_engine();

		for i in 0..100 {
			let moves = (engine.get_valid_moves(&engine.current_state.player1),
						 engine.get_valid_moves(&engine.current_state.player2));

			let random_move2 = *moves.1.iter().choose(&mut rng).unwrap();
			let random_move1 = *moves.0.iter().choose(&mut rng).unwrap();
			engine.apply_move((random_move1, random_move2));
		}
		println!("{:?}", engine.check_gameover());
	}
}
