use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::iter::FromIterator;
use std::path::Path;

use ndarray::Array;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand_distr::num_traits::Pow;
use rand_distr::num_traits::real::Real;
use tch::{CModule, Tensor};
use tch::nn::Module;

use engine::engine::Engine;
use engine::state::{Action, Direction, FruitType, WinState, SAR, PlayerWinner, MatchReplay};
use engine::state::Direction::Up;

use crate::manager::RolloutConfig;
use rand::thread_rng;
use std::time::Instant;
use crate::ai::{RandomPlayer, AiPlayer};

pub struct RolloutWorker<'a> {
	conf: RolloutConfig,
	engine: Engine,
	model_store: ModelStore<'a>,
	matchmaking: MatchmakingPool,
	match_history: Vec<MatchReplay>,
	sar_store: Vec<SAR>
}

pub struct ModelStore<'a> {
	pub root_path: &'a Path,
	pub model_ids: Vec<i32>,
	pub models_hash: HashMap<i32, CModule>,
}

pub enum PlayerIdx {
	Player1,
	Player2,
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
			match_history: Vec::new(),
			sar_store: Vec::new()
		}
	}

	pub fn play_match_agents(self: &mut Self) {
		let agent_ids = self.matchmaking.sample_pair();
		let start = Instant::now();
		self.reset();
		while self.engine.current_state.match_status == WinState::InProgress {
			let states = (
				self.get_state_vec_view(PlayerIdx::Player1),
				self.get_state_vec_view(PlayerIdx::Player2)
			);
			let actions =
				(
					self.run_model(agent_ids.0, states.0.0, states.0.1, self.conf.evaluation_mode),
					self.run_model(agent_ids.1, states.1.0, states.1.1, self.conf.evaluation_mode)
				);
			self.engine.apply_move(actions, Some(-0.1f32));

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
			let state = self.get_state_vec_view(PlayerIdx::Player1);
			let player_action = self.run_model(player_id.clone(), state.0, state.1, self.conf.evaluation_mode);
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
		self.sar_store.append(&mut self.engine.game_history);
	}

	pub fn play_matches(self: &mut Self) -> Vec<MatchReplay> {
		while self.sar_store.len() < self.conf.max_sars as usize {
			self.play_match_ai();
		}
		self.match_history.clone()
	}

	pub fn run_model(self: &Self, model_idx: i32, state_vec: Vec<f32>, action_mask: Vec<f32>,
					 evaluation_mode: bool) -> Action {
		let state_tensor = Tensor::of_slice(&state_vec);
		let mut action_vector = Tensor::of_slice(&action_mask);
		action_vector = action_vector * Tensor::from(-2.0f32);

		let pred = self.model_store.models_hash.get(&model_idx).unwrap().forward(&state_tensor) + action_vector;

		let action_idx = if evaluation_mode {
			i32::from(pred.argmax(0, false))
		} else {
			i32::from(pred.softmax(0, tch::Kind::Float).multinomial(1, true))
		};

		match action_idx {
			0 => Action::Move(Direction::Up),
			1 => Action::Move(Direction::Down),
			2 => Action::Move(Direction::Left),
			3 => Action::Move(Direction::Right),
			_ => Action::DoNothing,
		}
	}

	pub fn get_state_vec_view(self: &Self, idx: PlayerIdx) -> (Vec<f32>, Vec<f32>) {
		let action_mask = match idx {
			PlayerIdx::Player1 => {
				self.engine.get_valid_moves(&self.engine.current_state.player1)
			}
			PlayerIdx::Player2 => {
				self.engine.get_valid_moves(&self.engine.current_state.player2)
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

		let fruit_map = &self.engine.current_state.board.fruit_map;
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
		let player1 = &self.engine.current_state.player1;
		let player2 = &self.engine.current_state.player2;
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
