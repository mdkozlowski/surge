#[allow(dead_code)]
pub mod engine {
	use std::{collections::{HashMap, VecDeque}, convert::TryFrom};
	use rand::Rng;
	use rand::rngs::ThreadRng;
	use rand::seq::SliceRandom;
	use std::{iter, ops};
	use std::collections::HashSet;
	
	use ndarray::{Array, array, stack};

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
				Direction::Up => Position::new(0,-1),
				Direction::Down => Position::new(0,1),
				Direction::Left => Position::new(-1,0),
				Direction::Right => Position::new(1,0)
			}
		}
	}

	impl ops::Add<Position> for Position {
		type Output = Position;
	
		fn add(self, _rhs: Position) -> Position {
			Position {
				x: self.x + _rhs.x,
				y: self.y + _rhs.y
			}
		}
	}

	#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
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
		x: i8,
		y: i8
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
				y
			}
		}
	}

	#[derive(Debug)]
	pub struct Player {
		fruit_counts: HashMap<FruitType, i8>,
		position: Position,

	}

	#[derive(Debug)]
	pub struct Engine {
		game_history: Vec<SAR>,
		pub current_state: GameState
	}

	pub struct EngineConfig {
		pub board_size: i8,
		pub fruit_density: f32
	}

	#[derive(Debug)]
	pub struct BoardState {
		board_fruit: ndarray::ArrayBase<ndarray::OwnedRepr<std::option::Option<FruitType>>, ndarray::Dim<[usize; 2]>>,
		board_size: i8
	}
	#[derive(Debug)]
	pub struct GameState {
		players: Vec<Player>,
		board_state: BoardState,
		round: u32
	}
	
	#[derive(Debug)]
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
				game_history: vec![],
				current_state: GameState {
					players,
                    board_state,
                    round: 0
				}
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
					if !outside_bounds(self.current_state.board_state.board_size, target_pos.x) 
						&& !outside_bounds(self.current_state.board_state.board_size, target_pos.y) {
						valid_moves.insert(Action::Move(direction));
					}
				}
				valid_moveset.push(valid_moves);
			}
			valid_moveset
		}
		
		pub fn print_state(self: &Self) {
			let board_state = &self.current_state.board_state.board_fruit;
			for x in 0..10 {
				for y in 0..10 {
					let val = board_state[[x,y]];
					match val {
						None => print!("#"),
						Some(fruit) => match fruit {
							FruitType::Apple => print!("A"),
							FruitType::Banana => print!("B"),
							FruitType::Orange => print!("C")
						}
					}
					if y == 9 {
						print!("\r\n");
					}
				}
			}

			println!("{:?}", board_state);

        }
        
        pub fn apply_action(actions: (Action, Action)) {
            
        }

		pub fn initialise_board(conf: EngineConfig) -> (BoardState, Vec<Player>) {
			let mut rng = rand::thread_rng();
			let mut board_fruit : ndarray::ArrayBase<ndarray::OwnedRepr<std::option::Option<FruitType>>, ndarray::Dim<[usize; 2]>> 
					= Array::from_elem((conf.board_size as usize,conf.board_size as usize), None);

			let mut board_positions: Vec<Position> = vec![];
			for x in 0..conf.board_size {
				for y in 0..conf.board_size {
					board_positions.push(Position::new(x, y));
				}
			}
			board_positions.shuffle(&mut rng);
			let mut board_positions_queue = VecDeque::from(board_positions.clone());

			let total_fruit = conf.fruit_density * (conf.board_size.pow(2) as f32);
			let per_fruit = (total_fruit / 3.0f32).ceil() as usize;

			let mut fruit_values: Vec<FruitType> = 
				iter::repeat(FruitType::Apple).take(per_fruit)
						.chain(iter::repeat(FruitType::Banana).take(per_fruit))
						.chain(iter::repeat(FruitType::Orange).take(per_fruit)).collect();
			fruit_values.shuffle(&mut rng);

			for (fruit, pos) in fruit_values.iter().zip(board_positions.iter()) {
				board_fruit[[pos.x as usize, pos.y as usize]] = Some(*fruit);
				board_positions_queue.pop_front();
			}

			let mut players: Vec<Player> = vec![];
			for _ in 0..2 {
				
				let mut fruit_counts = HashMap::new();
				fruit_counts.insert(FruitType::Apple, 0);
				fruit_counts.insert(FruitType::Banana, 0);
				fruit_counts.insert(FruitType::Orange, 0);

				let position = board_positions_queue.pop_front().unwrap();

				players.push(Player {
					fruit_counts,
					position
				});
			}

			let board_state = BoardState {
				board_fruit,
				board_size: conf.board_size
			};

			(board_state, players)
		}
	}


}