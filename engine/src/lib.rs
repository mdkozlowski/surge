pub mod engine {
	use std::{collections::{HashMap, VecDeque}, convert::TryFrom};
	use rand::Rng;
	use rand::rngs::ThreadRng;
	use rand::seq::SliceRandom;
	use std::iter;

	#[derive(Debug)]
	pub enum Direction {
		Up,
		Down,
		Left,
		Right,
    }

	#[derive(Debug)]
	pub enum Action {
		Move(Direction),
		DoNothing
    }

	// #[derive(PartialEq)]
	#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
	pub enum FruitType {
		Apple = 1,
		Banana = 2,
		Orange = 3
	}

	impl FruitType {
		fn as_num(foo: FruitType) -> u8 {
			foo as u8
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

	#[derive(Eq, Hash, PartialEq, Copy, Clone)]
	pub struct Position {
		x: u8,
		y: u8
	}

	impl Position {
		pub fn random_position(max: &u8, rng: &mut rand::rngs::ThreadRng) -> Position {
			Position {
				x: rng.gen_range(0..*max) as u8,
				y: rng.gen_range(0..*max) as u8,
			}
		}

		pub fn new(x: u8, y: u8) -> Self {
			Position {
				x,
				y
			}
		}
	}

	pub struct Player {
		fruit_counts: HashMap<FruitType, u8>,
		position: Position,

	}

	pub struct Engine {
		match_history: Vec<SAR>,
		current_state: GameState
	}

	pub struct EngineConfig {
		board_size: u8,
		fruit_density: f32
	}

	pub struct BoardState {
		board_fruit: HashMap<Position, Option<FruitType>>
	}
		
		pub impl BoardState {
				pub fn read_game() {
					
				}
		}
	
	pub struct GameState {
		players: Vec<Player>,
		board_state: BoardState
	}
	
	pub struct SAR {
		gamestate: GameState,
		actions: [Action; 2],
		reward: [f32; 2]
		}
	

	impl EngineConfig {
		pub fn default() -> EngineConfig {
			EngineConfig {
				board_size: 8,
				fruit_density: 0.3_f32
			}
		}
	}

	impl Engine {
		pub fn new(conf: EngineConfig) -> Engine {

			let (board_state, players) = Engine::initialise_board(conf);

			Engine {
				match_history: vec![],
				current_state: GameState {
					players,
					board_state
				}
			}
		}

		pub fn initialise_board(conf: EngineConfig) -> (BoardState, Vec<Player>) {
			let mut rng = rand::thread_rng();
			let mut board_fruit : HashMap<Position, Option<FruitType>> = HashMap::new();

			let mut board_positions: Vec<Position> = vec![];
			for x in 0..conf.board_size {
				for y in 0..conf.board_size {
					board_positions.push(Position::new(x, y));
				}
			}
			board_positions.shuffle(&mut rng);
			let board_positions_queue = VecDeque::from(board_positions.clone());

			let mut fruit_values: Vec<FruitType> = 
				iter::repeat(FruitType::Apple).take(5)
						.chain(iter::repeat(FruitType::Banana).take(5))
						.chain(iter::repeat(FruitType::Orange).take(5)).collect();
			fruit_values.shuffle(&mut rng);

			for (fruit, pos) in fruit_values.iter().zip(board_positions.iter()) {
				board_fruit.insert(*pos, Some(*fruit));
				board_positions_queue.pop_front();
			}

			let mut players: Vec<Player> = vec![];
			for i in 0..2 {
				
				let fruit_counts = HashMap::new();
				fruit_counts.insert(FruitType::Apple, 0);
				fruit_counts.insert(FruitType::Banana, 0);
				fruit_counts.insert(FruitType::Orange, 0);

				let position = board_positions_queue.pop_front().unwrap();

				players.push(Player {
					fruit_counts,
					position
				});

				board_fruit.insert(position, None);
			}

			for position in board_positions_queue.iter() {
				board_fruit.insert(*position, None);
			}

			let board_state = BoardState {
				board_fruit
			};

			(board_state, players)
		}
	}


}