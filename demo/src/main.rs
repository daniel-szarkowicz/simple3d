use std::time::Instant;

use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::*;
use graphics::math::Transform;
use graphics::mesh::{MeshProvider, PNVertex, Static};
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
        canvas.draw(StaticLowPoly(Ellipsoid)).rotate_y(t);
        canvas
            .draw(ParametricSquare::new(500, |x, z| {
                let a = AutoGrad::new(x * 5.0, [1.0, 0.0]);
                let b = AutoGrad::new(z * 5.0, [0.0, 1.0]);
                let mut c = (a * 10.0.into() + (t * 5.0).into()).sin()
                    + (b * 100.0.into() + t.into()).sin();
                c = c / 10.0.into();
                // let c = AutoGrad::from(1.0) % (a * b);
                (c.val(), c.grad()[0], c.grad()[1])
            }))
            .scale(10.0, 1.0, 10.0)
            .translate(0.0, 0.0, 10.0);
        canvas
            .draw(RemCanyon)
            .scale(10.0, 1.0, 10.0)
            .translate(10.0, 0.0, 0.0);
    }
}

struct RemCanyon;

impl MeshProvider for RemCanyon {
    type Vertex = PNVertex;

    type Kind = Static;

    fn create_mesh(self) -> graphics::mesh::Mesh<Self::Vertex> {
        LowPoly(ParametricSquare::new(1200, |x, z| {
            let a = AutoGrad::new(x * 10.0, [1.0, 0.0]);
            let b = AutoGrad::new(z * 10.0, [0.0, 1.0]);
            let c = AutoGrad::from(1.0) % (a * b);
            (c.val(), c.grad()[0], c.grad()[1])
        }))
        .create_mesh()
    }
}
