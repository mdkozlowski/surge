use engine::engine::*;
use game_runner::worker::{RolloutWorker, ModelStore};
use game_runner::RunnerConfig;
use game_runner::worker::PlayerIdx::{Player1, Player2};

fn main() {
    let engine = Engine::new(EngineConfig::default());
    // engine.current_state.print_state();

    let mut model_store = ModelStore::new(&"A:\\surge\\model_store\\fc_model",
                                          vec![1, 2, 3]);
    let mut worker = RolloutWorker::new(RunnerConfig {
        engine_config: EngineConfig {
            random_seed: 134,
            fruit_density: 0.2f32,
            populate_board: true,
            board_size: 10
        }
    }, model_store);
    worker.reset();
    println!("{:?}", worker.get_state_vec_view(Player2));
}
