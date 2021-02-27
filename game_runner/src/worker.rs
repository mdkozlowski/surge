use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::iter::FromIterator;
use std::path::Path;
use std::time::Instant;

use ndarray::Array;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand_distr::num_traits::Pow;
use rand_distr::num_traits::real::Real;
use tch::{CModule, Tensor, IndexOp};
use tch::nn::Module;

use engine::engine::Engine;
use engine::state::{Action, Direction, FruitType, GameState, MatchReplay, PlayerIdx, PlayerWinner, SAR, WinState};
use engine::state::Direction::Up;

use crate::ai::{AiPlayer, RandomPlayer};
use crate::manager::RolloutConfig;
use std::process::exit;

pub struct RolloutWorker<'a> {
	conf: RolloutConfig,
	engine: Engine,
	model_store: ModelStore<'a>,
	matchmaking: MatchmakingPool,
	match_history: Vec<MatchReplay>
}

pub struct ModelStore<'a> {
	pub root_path: &'a Path,
	pub model_ids: Vec<i32>,
	pub models_hash: HashMap<i32, CModule>,
}

impl<'a> ModelStore<'a> {
	pub fn new(path: &'a str, model_ids: Vec<i32>) -> Self {
		let root_path = &Path::new(path);
		let mut models_hash = HashMap::<i32, CModule>::new();
		for id in &model_ids {
			let current = format!("{}.pt", id.to_string().as_str());
			let model_path = root_path.join(&Path::new(current.as_str()));
			let model = ModelStore::load_model(model_path.to_str().unwrap());
			models_hash.insert(*id, model);
		}

		ModelStore {
			root_path,
			model_ids,
			models_hash,
		}
	}

	fn load_model(model_path: &str) -> CModule {
		// println!("{}", model_path);
		let model = tch::CModule::load(model_path).unwrap_or_else(|_| {
			panic!("fuck");
		});
		model
	}
}

impl<'a> RolloutWorker<'a> {
	pub fn new(conf: RolloutConfig, model_store: ModelStore<'a>) -> Self {
		let engine = Engine::new(conf.engine_config.clone());
		RolloutWorker {
			matchmaking: MatchmakingPool {
				target_id: (&conf.agent_ids.0).clone(),
				opponent_ids: (&conf.agent_ids.1).clone(),
				rng: thread_rng(),
			},
			conf,
			engine,
			model_store,
			match_history: Vec::new()
		}
	}

	pub fn play_match_agents(self: &mut Self) {
		let agent_ids = self.matchmaking.sample_pair();
		let start = Instant::now();
		self.reset();
		while self.engine.current_state.match_status == WinState::InProgress {
			let states = (
				GameState::get_state_vec_view(&self.engine.current_state, PlayerIdx::Player1),
				GameState::get_state_vec_view(&self.engine.current_state, PlayerIdx::Player2)
			);
			let actions =
				(
					self.run_model(agent_ids.0, states.0.0, states.0.1, self.conf.evaluation_mode),
					self.run_model(agent_ids.1, states.1.0, states.1.1, self.conf.evaluation_mode)
				);
			self.engine.apply_move((actions.0.0, actions.1.0), Some(-0.1f32));

			if self.engine.current_state.round >= self.conf.max_rounds as u32 {
				break;
			}
		}
		let winstate = self.engine.current_state.match_status;
		println!("{:?}", winstate);
		let duration = start.elapsed();

		println!("Time elapsed in expensive_function() is: {:?}", duration);
		// self.sar_store.append(&mut self.engine.game_history);
		// self.win_history.push((agent_ids.0, agent_ids.1, winstate == WinState::Finished(PlayerWinner::Player1)))
	}

	pub fn play_match_ai(self: &mut Self) {
		let player_id = self.matchmaking.target_id.clone();
		let mut opponent = RandomPlayer::new();

		self.reset();
		while self.engine.current_state.match_status == WinState::InProgress {
			let state = GameState::get_state_vec_view(&self.engine.current_state, PlayerIdx::Player1);
			let (player_action, player_value) = self.run_model(player_id.clone(), state.0, state.1, self.conf.evaluation_mode);
			let opponent_action = opponent.get_move(&self.engine.current_state);
			self.engine.apply_move((player_action, opponent_action), Some(-0.1f32));

			if self.engine.current_state.round >= self.conf.max_rounds as u32 {
				break;
			}
		}
		let winstate = self.engine.current_state.match_status;

		let match_replay = MatchReplay {
			sars: self.engine.game_history.clone(),
			agent_ids: (player_id, 69420),
			p1_won: winstate == WinState::Finished(PlayerWinner::Player1)
		};
		self.match_history.push(match_replay);
		// self.sar_store.append(&mut self.engine.game_history);
	}

	pub fn play_matches(self: &mut Self) -> Vec<MatchReplay> {
		while self.match_history.len() < self.conf.max_matches as usize {
			self.play_match_ai();
		}
		self.match_history.clone()
	}

	pub fn run_model(self: &Self, model_idx: i32, state_vec: Vec<f32>, action_mask: Vec<f32>,
					 evaluation_mode: bool) -> (Action, f32) {
		let state_tensor = Tensor::of_slice(&state_vec);
		let mut action_vector = Tensor::of_slice(&action_mask);
		action_vector = action_vector * Tensor::from(-2.0f32);

		let pred = self.model_store.models_hash.get(&model_idx).unwrap().forward(&state_tensor);
		let mut action_pred = pred.i(0..4);
		let value_pred = pred.i(4);

		let action_pred = action_pred + action_vector;
		let action_idx = if evaluation_mode {
			i32::from(action_pred.argmax(0, false))
		} else {
			i32::from(action_pred.softmax(0, tch::Kind::Float).multinomial(1, true))
		};

		let action = match action_idx {
			0 => Action::Move(Direction::Up),
			1 => Action::Move(Direction::Down),
			2 => Action::Move(Direction::Left),
			3 => Action::Move(Direction::Right),
			_ => Action::DoNothing,
		};
		(action, f32::from(value_pred))
	}

	pub fn reset(&mut self) {
		self.engine = Engine::new(self.conf.engine_config.clone());
		// get new matchmaking settings
		// load new models
	}
}

pub struct MatchmakingPool {
	target_id: i32,
	opponent_ids: Vec<i32>,
	rng: ThreadRng,
}

impl MatchmakingPool {
	pub fn new(target: i32, opponents: Vec<i32>) -> Self {
		let mut thread_rng = ThreadRng::default();
		Self {
			target_id: target,
			opponent_ids: opponents,
			rng: thread_rng,
		}
	}

	pub fn sample_pair(self: &mut Self) -> (i32, i32) {
		let opponent = self.opponent_ids.choose(&mut self.rng).unwrap();
		(self.target_id.clone(), opponent.clone())
	}
}
