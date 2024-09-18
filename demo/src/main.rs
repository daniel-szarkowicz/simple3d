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
        canvas
            .draw(ParametricSquare::new(300, |x, z| {
                let a = AutoGrad::new(x * 10.0, [1.0, 0.0]);
                let b = AutoGrad::new(z * 10.0, [0.0, 1.0]);
                // let mut c = (a * 10.0.into() + (t * 5.0).into()).sin()
                //     * (b * 100.0.into() + t.into()).sin();
                // c = c / 20.0.into();
                let c = AutoGrad::from(1.0) % (a * b);
                (c.val(), c.grad()[0], c.grad()[1])
            }))
            .scale(10.0, 1.0, 10.0);
    }
}
