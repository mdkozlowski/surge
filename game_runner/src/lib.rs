use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use engine::engine::{Engine, EngineConfig};
use worker::RolloutWorker;

mod tests;
pub mod manager;
pub mod worker;

pub struct Runner<'a> {
	instances: Vec<RolloutWorker<'a>>,

}

pub struct MatchmakingPool {
	target_id: i32,
	opponent_ids: Vec<i32>,
	rng: ThreadRng,
}

pub struct RunnerConfig {
	pub engine_config: EngineConfig,

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