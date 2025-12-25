mod graphics;

use graphics::GraphicsEngine;

fn main() {
    let entry = graphics::create_entry();
    let _ = GraphicsEngine::new(&entry, c"ash-2dgame-template", 0);
}
