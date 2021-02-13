mod tests;

use engine::engine::{Engine, EngineConfig};
use rand::rngs::ThreadRng;
use rand::Rng;
use rand::seq::SliceRandom;

pub struct EngineInstance {
	conf: RunnerConfig,
	engine: Engine,
}

pub struct Runner {
	instances: Vec<EngineInstance>,

}

pub struct MatchmakingPool {
	target_id: i32,
	opponent_ids: Vec<i32>,
	rng: ThreadRng,
}

pub struct RunnerConfig {
	engine_config: EngineConfig,

}

impl EngineInstance {
	pub fn new(conf: RunnerConfig) -> Self {
		let engine = Engine::new(conf.engine_config.clone());
		EngineInstance {
			conf,
			engine,
		}
	}

	pub fn reset(mut self) {
		self.engine = Engine::new(conf.engine_config.clone());
	}
}

impl MatchmakingPool {
	pub fn new(target: i32, opponents: Vec<i32>) -> Self {
		let mut thread_rng = ThreadRng::default();
		Self {
			target_id: target,
			opponent_ids: opponents,
			rng: thread_rng
		}
	}

	pub fn sample_pair(self: &mut Self) -> (i32, i32) {
		let opponent = self.opponent_ids.choose(&mut self.rng).unwrap();
		(self.target_id, *opponent)
	}
}