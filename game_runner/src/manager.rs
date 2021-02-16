use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use engine::engine::EngineConfig;

use crate::worker::RolloutWorker;

#[derive(Clone)]
pub struct RolloutConfig {
	pub engine_config: EngineConfig,
	pub agent_ids: (usize, Vec<usize>),
	pub max_rounds: u32
}

pub struct RolloutManager<'a>  {
	instances: Vec<RolloutWorker<'a>>,

}