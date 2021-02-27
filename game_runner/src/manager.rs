use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use engine::engine::EngineConfig;

use crate::worker::RolloutWorker;

#[derive(Clone)]
pub struct RolloutConfig {
	pub engine_config: EngineConfig,
	pub agent_ids: (i32, Vec<i32>),
	pub max_rounds: u32,
	pub evaluation_mode: bool,
	pub max_matches: u32
}

pub struct RolloutManager<'a>  {
	instances: Vec<RolloutWorker<'a>>,

}