use std::time::Instant;

use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::*;
use graphics::math::Transform;
use math::auto_grad::{AutoGrad, Float};

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
        canvas.draw(Box).rotate_y(t).translate_x(3.0);
        canvas.draw(BoxLines).rotate_y(t).translate_x(-3.0);
        canvas.draw(ParametricSquare::new(150, |x, y| {
            let a = AutoGrad::new(x, [1.0, 0.0]);
            let b = AutoGrad::new(y, [0.0, 1.0]);
            // let a = AutoDiff::new(a, 1.0);
            // let result = (a * 10.0 + t * 5.0).sin() / 20.0;
            // (result.val(), result.diff(), 0.0)
            let mut c = (a * 10.0.into() + (t * 5.0).into()).sin()
                + (b * 100.0.into()).sin();
            c = c / 20.0.into();
            (c.val(), c.grad()[0], c.grad()[1])
        }));
    }
}
