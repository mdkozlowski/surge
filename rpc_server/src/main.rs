use engine::engine::*;

fn main() {
    let engine = Engine::new(EngineConfig::default());
    engine.print_state();
}
