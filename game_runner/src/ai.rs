use engine::state::{GameState, Action, Direction};
use rand::seq::{SliceRandom, IteratorRandom};
use rand::rngs::ThreadRng;
use rand::thread_rng;

pub trait AiPlayer {
	fn get_move(self: &mut Self, current_state: &GameState) -> Action;
}

pub struct RandomPlayer {
	pub rng: ThreadRng
}

impl RandomPlayer {
	pub fn new() -> Self {
		RandomPlayer {
			rng: thread_rng()
		}
	}
}

impl AiPlayer for RandomPlayer {
	fn get_move(self: &mut Self, current_state: &GameState) -> Action {
		let available_actions = current_state
			.get_valid_moves(&current_state.player2);
		let available_actions_iter = available_actions
			.iter()
			.collect::<Vec<&Action>>();
		// let action = [Action::Move(Direction::Up),
		// 	Action::Move(Direction::Down),
		// 	Action::Move(Direction::Left),
		// 	Action::Move(Direction::Right)];
		**available_actions_iter.choose(&mut self.rng).unwrap()
	}
}