#![allow(dead_code)]

use std::{collections::HashMap, convert::TryFrom};
use std::ops;
use std::collections::HashSet;
use std::hash::Hash;
use std::iter::FromIterator;

use ndarray::Array;
use num_traits::Pow;
use rand::Rng;
use rand::rngs::ThreadRng;

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum Direction {
	Up = 0,
	Down = 1,
	Left = 2,
	Right = 3,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum WinState {
	InProgress,
	Finished(PlayerWinner)
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum PlayerWinner {
	Player1,
	Player2
}

impl Direction {
	pub fn as_pos(self: &Self) -> Position {
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

impl Action {
	pub fn as_num(a: Action) -> i32 {
		match a {
			Action::Move(dir) => {
				dir as i32
			}
			_ => { 0 }
		}
	}
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
	pub x: i8,
	pub y: i8,
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

#[derive(Debug, Clone)]
pub struct Player {
	pub fruit_counts: HashMap<FruitType, f32>,
	pub position: Position,
	pub reward: f32
}

#[derive(Debug, Clone)]
pub struct BoardState {
	pub fruit_map: ndarray::ArrayBase<ndarray::OwnedRepr<std::option::Option<FruitType>>, ndarray::Dim<[usize; 2]>>,
	pub size: i8,
	pub fruit_counts: HashMap<FruitType, usize>
}

#[derive(Debug, Clone)]
pub struct GameState {
	pub player1: Player,
	pub player2: Player,
	pub board: BoardState,
	pub round: u32,
	pub match_status: WinState
}

#[derive(Debug, Clone)]
pub struct SAR {
	pub gamestate: GameState,
	pub actions: Action,
	pub action_mask: HashSet<Action>,
	pub reward: f32,
	pub terminal: bool
}

#[derive(Debug, Clone)]
pub struct MatchReplay {
	pub sars: Vec<SAR>,
	pub agent_ids: (i32, i32),
	pub p1_won: bool
}

impl BoardState {
	pub fn set_fruit(&mut self, x: usize, y: usize, fruit: Option<FruitType>) {
		self.fruit_map[[x as usize, y as usize]] = fruit;
		self.update_fruit_counts();
	}

	pub fn get_fruit(&mut self, x: usize, y: usize) -> Option<FruitType> {
		self.fruit_map[[x as usize, y as usize]].clone()
	}

	pub fn update_fruit_counts(&mut self) {
		let board = &self.fruit_map;
		let mut fruit_counts = HashMap::<FruitType, usize>::new();

		for fruit_cell in board.iter() {
			match *fruit_cell {
				Some(fruit) => {
					let val = fruit_counts.entry(fruit).or_insert(0);
					*val += 1;
				}
				_ => {}
			}
		}
		self.fruit_counts = fruit_counts;
	}
}

impl Player {
	pub fn new(pos: Position) -> Self {
		let mut fruit_counts = HashMap::new();
		fruit_counts.insert(FruitType::Apple, 0.0f32);
		fruit_counts.insert(FruitType::Banana, 0.0f32);
		fruit_counts.insert(FruitType::Orange, 0.0f32);
		Player {
			position: pos,
			fruit_counts,
			reward: 0.0f32
		}
	}

	pub fn move_player(&mut self, new_pos: Position) {
		self.position = new_pos;
	}

	pub fn increment_fruit(&mut self, fruit: FruitType, amount: f32) {
		let fruit_ref = self.fruit_counts.get_mut(&fruit).unwrap();
		*fruit_ref += amount;
		self.reward += amount;
	}

	pub fn get_fruit_count(&self, fruit: FruitType) -> &f32 {
		self.fruit_counts.get(&fruit).unwrap()
	}
}

impl GameState {
	pub fn print_state(self: &Self) {
		let board_state = &self.board.fruit_map;
		println!("Round {}:", self.round);
		for y in 0..self.board.size {
			for x in 0..self.board.size {
				let mut print_char = "#";
				let val = board_state[[x as usize, y as usize]];
				match val {
					None => {
						if (x as i8, y as i8) == (self.player1.position.x, self.player1.position.y) {
							print_char = "1";
						} else if (x as i8, y as i8) == (self.player2.position.x, self.player2.position.y) {
							print_char = "2";
						}
					}
					Some(fruit) => match fruit {
						FruitType::Apple => print_char = "A",
						FruitType::Banana => print_char = "B",
						FruitType::Orange => print_char = "C"
					}
				}
				print!("{}", print_char);
				if x == self.board.size - 1 {
					print!("\r\n");
				}
			}
		}

		println!("Player 1 @ <{},{}>: {}|{}|{}", self.player1.position.x, self.player1.position.y,
				 self.player1.fruit_counts.get(&FruitType::Apple).unwrap(),
				 self.player1.fruit_counts.get(&FruitType::Banana).unwrap(),
				 self.player1.fruit_counts.get(&FruitType::Orange).unwrap());
		println!("Player 2 @ <{},{}>: {}|{}|{}", self.player2.position.x, self.player2.position.y,
				 self.player2.fruit_counts.get(&FruitType::Apple).unwrap(),
				 self.player2.fruit_counts.get(&FruitType::Banana).unwrap(),
				 self.player2.fruit_counts.get(&FruitType::Orange).unwrap());
		println!("{:?}", self.board.fruit_counts);
		println!("===================")
	}

	pub fn get_valid_moves(self: &Self, player: &Player) -> HashSet<Action> {
		let outside_bounds = |board_size, val| {
			val >= board_size || val < 0
		};
		let mut valid_moves: HashSet<Action> = HashSet::new();
		// valid_moves.insert(Action::DoNothing);

		for direction in vec![Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
			let target_pos = direction.as_pos() + player.position;
			if !outside_bounds(self.board.size, target_pos.x)
				&& !outside_bounds(self.board.size, target_pos.y) {
				valid_moves.insert(Action::Move(direction));
			}
		}
		valid_moves
	}

	pub fn outside_bounds(board_size: i8, pos: &Position) -> bool {
		return (pos.x < 0 || pos.x >= board_size) || (pos.y < 0 || pos.y >= board_size);
	}

	pub fn get_state_vec_view(current_state: &GameState, idx: PlayerIdx) -> (Vec<f32>, Vec<f32>) {
		let action_mask = match idx {
			PlayerIdx::Player1 => {
				current_state.get_valid_moves(&current_state.player1)
			}
			PlayerIdx::Player2 => {
				current_state.get_valid_moves(&current_state.player2)
			}
		};
		let mut action_mask_array = Array::ones((4));
		for item in action_mask {
			match item {
				Action::Move(dir) => {
					match dir {
						Direction::Up => {
							action_mask_array[0] = 0.0f32;
						}
						Direction::Down => {
							action_mask_array[1] = 0.0f32;
						}
						Direction::Left => {
							action_mask_array[2] = 0.0f32;
						}
						Direction::Right => {
							action_mask_array[3] = 0.0f32;
						}
					}
				}
				_ => {}
			}
		}
		let action_mask_vec = Array::from_iter(action_mask_array.iter().cloned()).to_vec();

		let mut map = Array::zeros((10, 10, 3));

		let fruit_map = &current_state.board.fruit_map;
		for (idx, item) in fruit_map.indexed_iter() {
			match *item {
				Some(fruit) => {
					match fruit {
						FruitType::Apple => {
							map[[idx.0, idx.1, 0]] = 1.0f32;
						}
						FruitType::Banana => {
							map[[idx.0, idx.1, 1]] = 1.0f32;
						}
						FruitType::Orange => {
							map[[idx.0, idx.1, 2]] = 1.0f32;
						}
					}
				}
				None => {}
			}
		}
		let player1 = &current_state.player1;
		let player2 = &current_state.player2;
		let mut map_vec: Vec<f32> = Array::from_iter(map.iter().cloned()).to_vec();

		let mut own_info_vec: Vec<f32> = Vec::new();
		let mut their_info_vec: Vec<f32> = Vec::new();
		let mut relative_info_vec: Vec<f32> = Vec::new();
		let euclidean_dist = |x1: f32, x2: f32, y1: f32, y2: f32| -> f32 {
			(((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f32).sqrt() / 100.0f32
		};
		let manhattan_dist = |x1: f32, x2: f32, y1: f32, y2: f32| -> f32 {
			((x1 - x2).abs() + (y1 - y2) as f32).abs() / 100.0f32
		};
		let bearing_calc = |x1: f32, x2: f32, y1: f32, y2: f32| -> f32 {
			((y1 - y2).atan2(x1 - x2) + std::f32::consts::PI) / (2.0f32 * std::f32::consts::PI)
		};

		match idx {
			PlayerIdx::Player1 => {
				own_info_vec.push(*player1.fruit_counts.get(&FruitType::Apple).unwrap());
				own_info_vec.push(*player1.fruit_counts.get(&FruitType::Banana).unwrap());
				own_info_vec.push(*player1.fruit_counts.get(&FruitType::Orange).unwrap());
				own_info_vec.push(player1.position.x as f32 / 10.0f32);
				own_info_vec.push(player1.position.y as f32 / 10.0f32);

				their_info_vec.push(*player2.fruit_counts.get(&FruitType::Apple).unwrap());
				their_info_vec.push(*player2.fruit_counts.get(&FruitType::Banana).unwrap());
				their_info_vec.push(*player2.fruit_counts.get(&FruitType::Orange).unwrap());
				their_info_vec.push(player2.position.x as f32 / 10.0f32);
				their_info_vec.push(player2.position.y as f32 / 10.0f32);

				let euclidean_distance: f32 = euclidean_dist(player1.position.x as f32, player2.position.x as f32, player1.position.y as f32, player2.position.y as f32);
				let manhattan_distance: f32 = manhattan_dist(player1.position.x as f32, player2.position.x as f32, player1.position.y as f32, player2.position.y as f32);
				let bearing = bearing_calc(player1.position.x as f32, player2.position.x as f32, player1.position.y as f32, player2.position.y as f32);
				relative_info_vec.push(euclidean_distance);
				relative_info_vec.push(manhattan_distance);
				relative_info_vec.push(bearing.sin());
				relative_info_vec.push(bearing.cos());
			}
			PlayerIdx::Player2 => {
				own_info_vec.push(*player2.fruit_counts.get(&FruitType::Apple).unwrap());
				own_info_vec.push(*player2.fruit_counts.get(&FruitType::Banana).unwrap());
				own_info_vec.push(*player2.fruit_counts.get(&FruitType::Orange).unwrap());
				own_info_vec.push(player2.position.x as f32 / 10.0f32);
				own_info_vec.push(player2.position.y as f32 / 10.0f32);

				their_info_vec.push(*player1.fruit_counts.get(&FruitType::Apple).unwrap());
				their_info_vec.push(*player1.fruit_counts.get(&FruitType::Banana).unwrap());
				their_info_vec.push(*player1.fruit_counts.get(&FruitType::Orange).unwrap());
				their_info_vec.push(player1.position.x as f32 / 10.0f32);
				their_info_vec.push(player1.position.y as f32 / 10.0f32);

				let euclidean_distance: f32 = euclidean_dist(player2.position.x as f32, player1.position.x as f32, player2.position.y as f32, player1.position.y as f32);
				let manhattan_distance: f32 = manhattan_dist(player2.position.x as f32, player1.position.x as f32, player2.position.y as f32, player1.position.y as f32);
				let bearing = bearing_calc(player2.position.x as f32, player1.position.x as f32, player2.position.y as f32, player1.position.y as f32);
				relative_info_vec.push(euclidean_distance);
				relative_info_vec.push(manhattan_distance);
				relative_info_vec.push(bearing.sin());
				relative_info_vec.push(bearing.cos());
			}
		}

		map_vec.append(&mut own_info_vec);
		map_vec.append(&mut their_info_vec);
		map_vec.append(&mut relative_info_vec);

		(map_vec, action_mask_vec)
	}
}

pub enum PlayerIdx {
	Player1,
	Player2,
}
