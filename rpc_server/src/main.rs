use engine::engine::*;

fn main() {
    let engine = Engine::new(EngineConfig {
        board_size: 10,
        fruit_density: 0.25f32,
    });
    engine.print_state();
}
