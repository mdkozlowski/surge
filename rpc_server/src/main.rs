use engine::engine::*;

fn main() {
    let engine = Engine::new(EngineConfig::default());
    engine.current_state.print_state();
}
