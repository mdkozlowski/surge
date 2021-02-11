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
	use std::io::SeekFrom::End;
	use std::env::current_dir;
	use crate::state::PlayerWinner::Player2;
	use crate::state::FruitType::{Apple, Banana, Orange};

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

		pub fn get_valid_moves(self: &Self, player: &Player) -> HashSet<Action> {
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

		pub(crate) fn apply_move(&mut self, actions: (Action, Action)) -> WinState {
			let state = self.current_state.clone();
			let actions_copy = actions.clone();

			self.resolve_actions(actions);
			let new_rewards = Engine::make_reward_zerosum(
				self.current_state.player1.reward, self.current_state.player2.reward);
			self.current_state.round += 1;

			let sar: SAR = SAR {
				reward: [new_rewards.0, new_rewards.1],
				actions: [actions_copy.0, actions_copy.1],
				gamestate: state,
			};
			self.game_history.push(sar);

			let win_state = self.check_gameover();
			win_state
		}

		fn check_gameover(self: &Self) -> WinState {
			let most_fruit_counts = self.current_state.board.fruit_counts.values().max();
			match most_fruit_counts {
				Some(count) => {
					if *count == 0 {
						let player_winner = self.get_winner();
						WinState::Finished(player_winner)
					} else {
						WinState::InProgress
					}
				}
				None => {
					let player_winner = self.get_winner();
					WinState::Finished(player_winner)
				}
			}
		}

		fn get_winner(self: &Self) -> PlayerWinner {
			let player1_fruit = &self.current_state.player1.fruit_counts;
			let player2_fruit = &self.current_state.player2.fruit_counts;

			// count in how many fruit categories did player1 win
			let mut orange_gt = 0;
			let mut apple_gt = 0;
			let mut banana_gt =0;

			if player1_fruit.get(&Apple).unwrap() > player2_fruit.get(&Apple).unwrap() {
				apple_gt = 1;
			}
			if player1_fruit.get(&Banana).unwrap() > player2_fruit.get(&Banana).unwrap() {
				banana_gt = 1;
			}
			if player1_fruit.get(&Orange).unwrap() > player2_fruit.get(&Orange).unwrap() {
				orange_gt = 1;
			}

			let player1_wins = apple_gt + banana_gt + orange_gt;
			// let player1_wins = player1_fruit.values()
			// 	.zip(player2_fruit.values())
			// 	.map(|(a, b)| a > b)
			// 	.filter(|a: &bool| *a)
			// 	.count();

			match player1_wins {
				2 | 3 => PlayerWinner::Player1,
				_ => PlayerWinner::Player2
			}
		}

		fn make_reward_zerosum(reward1: f32, reward2: f32) -> (f32, f32) {
			let reward1_new = reward1 - reward2;
			let reward2_new = reward2 - reward1;

			(reward1_new, reward2_new)
		}

		fn resolve_actions(&mut self, actions: (Action, Action)) {
			//let p1_actions = self.get_valid_moves(&self.current_state.player1);
			//let p2_actions = self.get_valid_moves(&self.current_state.player2);

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

				let mut count = board.fruit_counts.get_mut(&fruit).unwrap();
				*count -= 1;
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

			let mut fruit_counts_map = HashMap::<FruitType, usize>::new();
			for fruit_type in [Apple, Banana, Orange].iter() {
				fruit_counts_map.insert(*fruit_type, 0);
			}

			let mut board_positions: Vec<Position> = vec![];
			for x in 0..conf.board_size {
				for y in 0..conf.board_size {
					board_positions.push(Position::new(x, y));
				}
			}
			board_positions.shuffle(&mut rng);
			let mut board_positions_queue = VecDeque::from(board_positions.clone());

			let mut player1 = Player::new(Position::new(0, 0));
			let mut player2 = Player::new(Position::new(1, 1));

			if conf.populate_board {
				let total_fruit = (conf.fruit_density * ((conf.board_size as u32).pow(2) as f32)).ceil();
				let dirichlet = Dirichlet::new_with_size(5.0f32, 3).unwrap();
				let proportions = dirichlet.sample(&mut rng);
				let mut fruit_counts = proportions.iter()
					.map(|a| (a * total_fruit).round() as usize)
					.map(|a| a + 1 - (a % 2))
					.collect::<Vec<usize>>();

				let mut fruit_values = &mut fruit_counts.iter()
					.zip([Apple, Orange, Banana].iter().copied())
					.flat_map(|(count, fruit)| iter::repeat(fruit).take(*count))
					.collect::<Vec<FruitType>>();

				fruit_values.shuffle(&mut rng);

				for (fruit, pos) in fruit_values.iter().zip(board_positions.iter()) {
					board_fruit[[pos.x as usize, pos.y as usize]] = Some(*fruit);
					board_positions_queue.pop_front();
				}

				player1.position = board_positions_queue.pop_front().unwrap();
				player2.position = board_positions_queue.pop_front().unwrap();

				for (count, fruit_type) in fruit_counts.iter().zip([Apple, Banana, Orange].iter().copied()) {
					let mut fruit_ref = fruit_counts_map.get_mut(&fruit_type).unwrap();
					*fruit_ref += count;
				}
			}

			let board_state = BoardState {
				fruit_map: board_fruit,
				size: conf.board_size,
				fruit_counts: fruit_counts_map,
			};

			(board_state, player1, player2)
		}
	}
}