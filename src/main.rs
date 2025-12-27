mod graphics;
mod settings;

use graphics::GraphicsEngine;

fn main() {
    let gengine = GraphicsEngine::new();

    loop {
        if !gengine.run() {
            break;
        }
    }
}
