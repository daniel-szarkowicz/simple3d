use std::time::Instant;

use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::*;
use graphics::math::Transform;

fn main() {
    App::run_with(State::new());
}

struct State {
    start: Instant,
}

impl State {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl AppState for State {
    fn update(&mut self) {}

    fn draw(&self, canvas: &mut Canvas) {
        let t = self.start.elapsed().as_secs_f32();
        let size = t.sin();
        canvas.draw(Ellipsoid).scale(size, size, size);
        canvas.draw(Box).scale(0.1, 0.1, 0.1).color([1.0, 0.0, 0.0]);
        canvas.draw(Box).rotate_y(t).translate_x(3.0);
        canvas.draw(BoxLines).rotate_y(t).translate_x(-3.0);
    }
}
