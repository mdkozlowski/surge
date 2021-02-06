mod tests;

#[allow(dead_code)]
pub mod engine {
	use std::{collections::{HashMap, VecDeque}, convert::TryFrom};
	use rand::rngs::{ThreadRng, StdRng};
	use rand::seq::SliceRandom;
	use std::{iter, ops};
	use std::iter::Enumerate;
	use std::collections::HashSet;

	use ndarray::{Array, array, stack};
	use rand::{Rng, SeedableRng};
	use std::io::SeekFrom::End;

	#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
	pub enum Direction {
		Up,
		Down,
		Left,
		Right,
	}

	impl Direction {
		fn as_pos(self: &Self) -> Position {
			match self {
				Direction::Up => Position::new(0, -1),
				Direction::Down => Position::new(0, 1),
				Direction::Left => Position::new(-1, 0),
				Direction::Right => Position::new(1, 0)
			}
		}
	}

	impl ops::Add<Position> for Position {
		type Output = Position;

		fn add(self, _rhs: Position) -> Position {
			Position {
				x: self.x + _rhs.x,
				y: self.y + _rhs.y,
			}
		}
	}

	#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
	pub enum Action {
		Move(Direction),
		DoNothing,
	}

	// #[derive(PartialEq)]
	#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
	pub enum FruitType {
		Apple = 1,
		Banana = 2,
		Orange = 3,
	}

	impl FruitType {
		fn as_num(foo: FruitType) -> i8 {
			foo as i8
		}

		fn random_fruit(rng: &mut ThreadRng) -> FruitType {
			FruitType::try_from(rng.gen_range(1..=3)).unwrap()
		}
	}

	impl TryFrom<i32> for FruitType {
		type Error = ();

		fn try_from(v: i32) -> Result<Self, Self::Error> {
			match v {
				x if x == FruitType::Apple as i32 => Ok(FruitType::Apple),
				x if x == FruitType::Banana as i32 => Ok(FruitType::Banana),
				x if x == FruitType::Orange as i32 => Ok(FruitType::Orange),
				_ => Err(()),
			}
		}
	}

	#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
	pub struct Position {
		pub(crate) x: i8,
		pub(crate) y: i8,
	}

	impl Position {
		pub fn random_position(max: &i8, rng: &mut rand::rngs::ThreadRng) -> Position {
			Position {
				x: rng.gen_range(0..*max) as i8,
				y: rng.gen_range(0..*max) as i8,
			}
		}

		pub fn new(x: i8, y: i8) -> Self {
			Position {
				x,
				y,
			}
		}
	}

	#[derive(Debug)]
	pub struct Player {
		pub(crate) fruit_counts: HashMap<FruitType, i8>,
		pub(crate) position: Position,

	}

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

	#[derive(Debug)]
	pub struct BoardState {
		fruit_map: ndarray::ArrayBase<ndarray::OwnedRepr<std::option::Option<FruitType>>, ndarray::Dim<[usize; 2]>>,
		size: i8,
	}

	#[derive(Debug)]
	pub struct GameState {
		pub players: Vec<Player>,
		pub board: BoardState,
		round: u32,
	}

	#[derive(Debug)]
	pub struct SAR {
		gamestate: GameState,
		actions: [Action; 2],
		reward: [f32; 2],
	}


	impl EngineConfig {
		pub fn default() -> EngineConfig {
			EngineConfig {
				board_size: 7,
				fruit_density: 0.3_f32,
				populate_board: true,
			}
		}
	}

	impl BoardState {
		pub fn set_fruit(&mut self, x: usize, y: usize, fruit: FruitType) {
			self.fruit_map[[x as usize, y as usize]] = Some(fruit);
		}

		pub fn get_fruit(&mut self, x: usize, y: usize) -> &Option<FruitType> {
			&self.fruit_map[[x as usize, y as usize]]
		}
	}

	impl GameState {
		pub fn insert_player(&mut self, p: Player) {
			self.players.push(p);
		}
	}

	impl Player {
		pub fn move_player(&mut self, new_pos: Position) {
			self.position = new_pos;
		}

		pub fn increment_fruit(&mut self, fruit: FruitType) {
			let mut fruit_ref = self.fruit_counts.get_mut(&fruit).unwrap();
			*fruit_ref += 1;
		}

		pub fn get_fruit_count(&self, fruit: FruitType) -> &i8 {
			self.fruit_counts.get(&fruit).unwrap()
		}
	}

	impl Engine {
		pub fn new(conf: EngineConfig) -> Engine {
			let (board_state, players) = Engine::initialise_board(conf);

			Engine {
				game_history: vec![],
				current_state: GameState {
					players,
					board: board_state,
					round: 0,
				},
			}
		}

		pub fn get_valid_moves(self: &Self) -> Vec<HashSet<Action>> {
			let mut valid_moveset: Vec<HashSet<Action>> = vec![];
			let outside_bounds = |board_size, val| {
				val >= board_size || val < 0
			};


			let players = &self.current_state.players;
			for player in players {
				let mut valid_moves: HashSet<Action> = HashSet::new();
				valid_moves.insert(Action::DoNothing);

				for direction in vec![Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
					let target_pos = direction.as_pos() + player.position;
					if !outside_bounds(self.current_state.board.size, target_pos.x)
						&& !outside_bounds(self.current_state.board.size, target_pos.y) {
						valid_moves.insert(Action::Move(direction));
					}
				}
				valid_moveset.push(valid_moves);
			}
			valid_moveset
		}

		pub fn print_state(self: &Self) {
			let board_state = &self.current_state.board.fruit_map;
			println!("Round {}:", self.current_state.round);
			for y in 0..self.current_state.board.size {
				for x in 0..self.current_state.board.size {
					let mut print_char = "#";
					let players = &self.current_state.players;
					let val = board_state[[x as usize, y as usize]];
					match val {
						None => {
							for (idx, player) in players.iter().enumerate() {
								if (x as i8, y as i8) == (player.position.x, player.position.y) {
									if idx == 0 {
										print_char = "1";
									} else {
										print_char = "2";
									}
								}
							}
						}
						Some(fruit) => match fruit {
							FruitType::Apple => print_char = "A",
							FruitType::Banana => print_char = "B",
							FruitType::Orange => print_char = "C"
						}
					}
					print!("{}", print_char);
					if x == self.current_state.board.size - 1 {
						print!("\r\n");
					}
				}
			}
			for (idx, player) in self.current_state.players.iter().enumerate() {
				println!("Player {} @ <{},{}>: {}|{}|{}", idx, player.position.x, player.position.y,
						 player.fruit_counts.get(&FruitType::Apple).unwrap(),
						 player.fruit_counts.get(&FruitType::Banana).unwrap(),
						 player.fruit_counts.get(&FruitType::Orange).unwrap())
			}
			println!("===================")
		}

		fn outside_bounds(board_size: i8, pos: &Position) -> bool {
			return (pos.x < 0 || pos.x >= board_size) || (pos.y < 0 || pos.y >= board_size);
		}

		pub fn apply_actions(&mut self, actions: (Action, Action)) {
			let mut p1_target = Engine::resolve_move(actions.0, &self.current_state.players.get(0).unwrap().position);
			let mut p2_target = Engine::resolve_move(actions.1, &self.current_state.players.get(1).unwrap().position);

			if &p1_target == &p2_target {
				return;
			}

			if Engine::outside_bounds(self.current_state.board.size, &p1_target) {
				p1_target = self.current_state.players.get(0).unwrap().position;
			}

			if Engine::outside_bounds(self.current_state.board.size, &p2_target) {
				p2_target = self.current_state.players.get(1).unwrap().position;
			}

			let mut players_ref = &mut self.current_state.players;
			let player1_ref = players_ref.get_mut(0).unwrap();
			player1_ref.move_player(p1_target);
			Engine::pickup_fruit(player1_ref, &mut self.current_state.board);

			let player2_ref = players_ref.get_mut(1).unwrap();
			player2_ref.move_player(p2_target);
			Engine::pickup_fruit(player2_ref, &mut self.current_state.board);
		}

		fn pickup_fruit(player: &mut Player, board: &mut BoardState) {
			let player_pos = &player.position;
			if let Some(fruit) = board.fruit_map[[player_pos.x as usize, player_pos.y as usize]] {
				board.fruit_map[[player_pos.x as usize, player_pos.y as usize]] = None;
				player.increment_fruit(fruit);
			}
		}

		pub fn resolve_move(action: Action, pos: &Position) -> Position {
			match action {
				Action::DoNothing => *pos,
				Action::Move(dir) => {
					dir.as_pos() + *pos
				}
			}
		}

		pub fn initialise_board(conf: EngineConfig) -> (BoardState, Vec<Player>) {
			let mut rng: StdRng = SeedableRng::seed_from_u64(1234);

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

			let mut players: Vec<Player> = vec![];
			if conf.populate_board {
				let total_fruit = conf.fruit_density * (conf.board_size.pow(2) as f32);
				let mut n_fruit = (total_fruit / 3.0f32).ceil() as usize;
				if n_fruit % 2 == 0 {
					n_fruit = n_fruit + 1;
				}

				let mut fruit_values: Vec<FruitType> =
					iter::repeat(FruitType::Apple).take(n_fruit)
						.chain(iter::repeat(FruitType::Banana).take(n_fruit))
						.chain(iter::repeat(FruitType::Orange).take(n_fruit)).collect();
				fruit_values.shuffle(&mut rng);

				for (fruit, pos) in fruit_values.iter().zip(board_positions.iter()) {
					board_fruit[[pos.x as usize, pos.y as usize]] = Some(*fruit);
					board_positions_queue.pop_front();
				}

				for _ in 0..2 {
					let mut fruit_counts = HashMap::new();
					fruit_counts.insert(FruitType::Apple, 0);
					fruit_counts.insert(FruitType::Banana, 0);
					fruit_counts.insert(FruitType::Orange, 0);

					let position = board_positions_queue.pop_front().unwrap();

					players.push(Player {
						fruit_counts,
						position,
					});
				}
			}

			let board_state = BoardState {
				fruit_map: board_fruit,
				size: conf.board_size,
			};

			(board_state, players)
		}
	}
}