#![allow(dead_code)]

use std::{collections::{HashMap}, convert::TryFrom};
use std::{ops};

use rand::{Rng};
use rand::rngs::{ThreadRng};

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum Direction {
	Up,
	Down,
	Left,
	Right,
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

#[derive(Debug)]
pub struct Player {
	pub fruit_counts: HashMap<FruitType, f32>,
	pub position: Position,

}

#[derive(Debug)]
pub struct BoardState {
	pub fruit_map: ndarray::ArrayBase<ndarray::OwnedRepr<std::option::Option<FruitType>>, ndarray::Dim<[usize; 2]>>,
	pub size: i8,
}

#[derive(Debug)]
pub struct GameState {
	pub player1: Player,
	pub player2: Player,
	pub board: BoardState,
	pub round: u32,
}

#[derive(Debug)]
pub struct SAR {
	gamestate: GameState,
	actions: [Action; 2],
	reward: [f32; 2],
}

impl BoardState {
	pub fn set_fruit(&mut self, x: usize, y: usize, fruit: Option<FruitType>) {
		self.fruit_map[[x as usize, y as usize]] = fruit;
	}

	pub fn get_fruit(&mut self, x: usize, y: usize) -> Option<FruitType> {
		self.fruit_map[[x as usize, y as usize]].clone()
	}
}

impl Player {
	pub fn move_player(&mut self, new_pos: Position) {
		self.position = new_pos;
	}

	pub fn increment_fruit(&mut self, fruit: FruitType, amount: f32) {
		let fruit_ref = self.fruit_counts.get_mut(&fruit).unwrap();
		*fruit_ref += amount;
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
		println!("===================")
	}
}
