use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use engine::engine::{Engine, EngineConfig};
use worker::RolloutWorker;

mod tests;
pub mod manager;
pub mod worker;