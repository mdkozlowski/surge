#[allow(dead_code)]
mod tests;

#[allow(dead_code)]
pub mod state;

#[allow(dead_code)]
pub mod engine {
	use std::{collections::{HashMap, VecDeque}};
	use std::{iter};
	use std::collections::HashSet;

	use ndarray::{Array};
	use rand::prelude::*;
	use rand::{SeedableRng};
	use rand::rngs::{StdRng};
	use rand::seq::SliceRandom;
	use rand_distr::Dirichlet;

	pub use crate::state::*;
	use std::slice::Iter;

	#[derive(Debug)]
	pub struct Engine {
		game_history: Vec<SAR>,
		pub current_state: GameState,
	}

	pub struct EngineConfig {
		pub board_size: i8,
		pub fruit_density: f32,
		pub populate_board: bool,
	}


	impl EngineConfig {
		pub fn default() -> EngineConfig {
			EngineConfig {
				board_size: 15,
				fruit_density: 0.2_f32,
				populate_board: true,
			}
		}
	}

	impl Engine {
		pub fn new(conf: EngineConfig) -> Engine {
			let (board_state, player1, player2) = Engine::initialise_board(conf);

			Engine {
				game_history: vec![],
				current_state: GameState {
					player1,
					player2,
					board: board_state,
					round: 0,
				},
			}
		}

		pub fn get_valid_moves(self: &Self, player: Player) -> HashSet<Action> {
			let outside_bounds = |board_size, val| {
				val >= board_size || val < 0
			};
			let mut valid_moves: HashSet<Action> = HashSet::new();
			valid_moves.insert(Action::DoNothing);

			for direction in vec![Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
				let target_pos = direction.as_pos() + player.position;
				if !outside_bounds(self.current_state.board.size, target_pos.x)
					&& !outside_bounds(self.current_state.board.size, target_pos.y) {
					valid_moves.insert(Action::Move(direction));
				}
			}
			valid_moves
		}

		fn outside_bounds(board_size: i8, pos: &Position) -> bool {
			return (pos.x < 0 || pos.x >= board_size) || (pos.y < 0 || pos.y >= board_size);
		}

		pub fn apply_actions(&mut self, actions: (Action, Action)) {
			let mut p1_target = Engine::resolve_move(actions.0, &self.current_state.player1.position);
			let mut p2_target = Engine::resolve_move(actions.1, &self.current_state.player2.position);

			if Engine::outside_bounds(self.current_state.board.size, &p1_target) {
				p1_target = self.current_state.player1.position.clone();
			}

			if Engine::outside_bounds(self.current_state.board.size, &p2_target) {
				p2_target = self.current_state.player2.position.clone();
			}

			if &p1_target == &p2_target {
				let board_fruit = self.current_state.board.get_fruit(p1_target.x as usize, p1_target.y as usize);
				match board_fruit {
					Some(fruit) => {
						self.current_state.board.set_fruit(p1_target.x as usize, p1_target.y as usize, None);
						&mut self.current_state.player1.increment_fruit(fruit, 0.5f32);
						&mut self.current_state.player2.increment_fruit(fruit, 0.5f32);
					}
					None => {}
				}
			} else {
				self.current_state.player1.move_player(p1_target);
				Engine::pickup_fruit(&mut self.current_state.player1, &mut self.current_state.board);

				self.current_state.player2.move_player(p2_target);
				Engine::pickup_fruit(&mut self.current_state.player2, &mut self.current_state.board);
			}
		}

		fn pickup_fruit(player: &mut Player, board: &mut BoardState) {
			let player_pos = &player.position;
			if let Some(fruit) = board.fruit_map[[player_pos.x as usize, player_pos.y as usize]] {
				board.fruit_map[[player_pos.x as usize, player_pos.y as usize]] = None;
				player.increment_fruit(fruit, 1.0f32);
			}
		}

		pub fn resolve_move(action: Action, pos: &Position) -> Position {
			match action {
				Action::DoNothing => *pos,
				Action::Move(dir) => dir.as_pos() + *pos
			}
		}

		pub fn initialise_board(conf: EngineConfig) -> (BoardState, Player, Player) {
			let mut rng: StdRng = SeedableRng::seed_from_u64(12345);

			let mut board_fruit: ndarray::ArrayBase<ndarray::OwnedRepr<std::option::Option<FruitType>>, ndarray::Dim<[usize; 2]>>
				= Array::from_elem((conf.board_size as usize, conf.board_size as usize), None);

			let mut board_positions: Vec<Position> = vec![];
			for x in 0..conf.board_size {
				for y in 0..conf.board_size {
					board_positions.push(Position::new(x, y));
				}
			}
			board_positions.shuffle(&mut rng);
			let mut board_positions_queue = VecDeque::from(board_positions.clone());

			let mut fruit_counts = HashMap::new();
			fruit_counts.insert(FruitType::Apple, 0.0f32);
			fruit_counts.insert(FruitType::Banana, 0.0f32);
			fruit_counts.insert(FruitType::Orange, 0.0f32);
			let mut player1 = Player {
				fruit_counts: fruit_counts.clone(),
				position: Position::new(0, 0),
			};

			let mut player2 = Player {
				fruit_counts,
				position: Position::new(1, 1),
			};

			if conf.populate_board {
				let total_fruit = (conf.fruit_density * ((conf.board_size as u32).pow(2) as f32)).ceil();
				let dirichlet = Dirichlet::new_with_size(5.0f32, 3).unwrap();
				let proportions = dirichlet.sample(&mut rng);
				let mut fruit_counts = proportions.iter()
					.map(|a| (a * total_fruit).round() as usize)
					.map(|a| a + 1 - (a % 2));

				let mut fruit_values = fruit_counts
					.zip([FruitType::Apple, FruitType::Orange, FruitType::Banana].iter().copied())
					.flat_map(|(count, fruit)| iter::repeat(fruit).take(count)).collect::<Vec<FruitType>>();

				fruit_values.shuffle(&mut rng);

				for (fruit, pos) in fruit_values.iter().zip(board_positions.iter()) {
					board_fruit[[pos.x as usize, pos.y as usize]] = Some(*fruit);
					board_positions_queue.pop_front();
				}

				player1.position = board_positions_queue.pop_front().unwrap();
				player2.position = board_positions_queue.pop_front().unwrap();
			}

			let board_state = BoardState {
				fruit_map: board_fruit,
				size: conf.board_size,
			};

			(board_state, player1, player2)
		}
	}
}