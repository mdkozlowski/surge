use engine::engine::Engine;

use crate::RunnerConfig;
use std::path::Path;
use ndarray::Array;
use std::iter::FromIterator;
use engine::state::FruitType;
use rand_distr::num_traits::Pow;
use rand_distr::num_traits::real::Real;
use tch::CModule;
use std::collections::HashMap;

pub struct RolloutWorker<'a> {
	conf: RunnerConfig,
	engine: Engine,
	model_store: ModelStore<'a>,
}

pub struct ModelStore<'a> {
	pub root_path: &'a Path,
	pub model_ids: Vec<usize>,
	pub models_hash: HashMap<usize, CModule>
}

pub enum PlayerIdx {
	Player1,
	Player2,
}

impl<'a> ModelStore<'a> {
	pub fn new(path: &'a str, model_ids: Vec<usize>) -> Self {
		let root_path = &Path::new(path);
		let mut models_hash = HashMap::<usize, CModule>::new();
		for id in &model_ids {
			let current = format!("{}.pt", id.to_string().as_str());
			let model_path = root_path.join(&Path::new(current.as_str()));
			let model = ModelStore::load_model(model_path.to_str().unwrap());
			models_hash.insert(*id, model);
		}

		ModelStore {
			root_path,
			model_ids,
			models_hash
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
 b
impl<'a> RolloutWorker<'a> {
	pub fn new(conf: RunnerConfig, model_store: ModelStore<'a>) -> Self {
		let engine = Engine::new(conf.engine_config.clone());
		RolloutWorker {
			conf,
			engine,
			model_store,
		}
	}

	pub fn get_state_vec_view(self: &Self, idx: PlayerIdx) -> Vec<f32> {
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
		map_vec
	}

	pub fn reset(&mut self) {
		self.engine = Engine::new(self.conf.engine_config.clone());
		// get new matchmaking settings
		// load new models
	}
}
