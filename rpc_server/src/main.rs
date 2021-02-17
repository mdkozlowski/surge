use engine::engine::*;
use game_runner::worker::{RolloutWorker, ModelStore};
use game_runner::manager::RolloutConfig;
use game_runner::worker::PlayerIdx::{Player1, Player2};

fn main() {
    println!("Start");
    let engine = Engine::new(EngineConfig::default());
    // engine.current_state.print_state();

    let model_store = ModelStore::new(&"A:\\surge\\model_store\\fc_model",
                                          vec![1, 2, 3]);
    let mut worker = RolloutWorker::new(RolloutConfig {
        engine_config: EngineConfig {
            random_seed: 11413,
            fruit_density: 0.2f32,
            populate_board: true,
            board_size: 10,
        },
        agent_ids: (1, vec![2,3]),
        max_rounds: 5_000,
        evaluation_mode: false
    }, model_store);
    worker.play_match_ai();
}
