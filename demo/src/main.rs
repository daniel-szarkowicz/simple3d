use std::time::Instant;

use engine::app::{App, AppState};
use engine::canvas::Canvas;
use engine::geometry::{Box, Ellipsoid};
use engine::math::Transform;

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
        let angle = t * 2.0 * std::f32::consts::PI / 5.0;
        canvas
            .group(|canvas| {
                canvas.draw(Ellipsoid).rotate_y(angle).translate_x(-2.0);
                canvas
                    .draw(Box)
                    .scale(1.5, 1.5, 1.5)
                    .rotate_y(angle)
                    .translate_x(2.0);
            })
            .rotate_y(angle / 2.0);
    }
}
