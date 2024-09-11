use std::time::Instant;

use engine::app::App;
use engine::canvas::Canvas;
use engine::geometry::{Box, Ellipsoid};
use engine::math::Transform;
// use engine::math::Transform;

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
    let count = 3;
    canvas
        .group(|canvas| {
            for i in 0..count {
                canvas
                    .draw(Box)
                    .scale_x((count - i) as f32)
                    .translate_y(i as f32);
            }
        })
        .rotate_y(angle);
}
