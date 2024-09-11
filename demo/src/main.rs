use std::time::Instant;

use engine::app::App;
use engine::canvas::Canvas;
use engine::geometry::{Box, Ellipsoid};
use engine::math::Transform;

fn main() {
    let app = App::new(State::new(), update, draw);

    app.run();
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

fn update(state: &mut State) {}

fn draw(canvas: &mut Canvas, state: &State) {
    let t = state.start.elapsed().as_secs_f32();
    let angle = t * 2.0 * std::f32::consts::PI / 10.0;
    canvas
        .group(|canvas| {
            canvas.draw(Ellipsoid).rotate_y(angle).translate_x(-2.0);
            canvas
                .draw(Box)
                .scale(1.5, 1.5, 1.5)
                .rotate_y(angle)
                .translate_x(2.0);
        })
        .rotate_y(-angle);
}
