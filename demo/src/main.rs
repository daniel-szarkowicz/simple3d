use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::*;
use graphics::math::Transform;
use nalgebra::Vector3;
use physics::{Shape, World};

fn main() {
    App::run_with(State::new());
}

struct State {
    world: World,
}

impl State {
    fn new() -> Self {
        let mut world = World::new();
        world
            .add_staticbody(Shape::Box {
                width: 2.0,
                height: 5.0,
                depth: 0.5,
            })
            .position(Vector3::new(0.0, -10.0, 0.0))
            .finish();
        Self { world }
    }
}

impl AppState for State {
    fn update(&mut self) {
        for mut sb in self.world.staticbodies_mut() {
            sb.position().y += 1.0 / 60.0 / 10.0;
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        for sb in self.world.staticbodies() {
            match sb.shape() {
                Shape::Sphere { diameter } => canvas.draw(Ellipsoid).scale(
                    *diameter as f32,
                    *diameter as f32,
                    *diameter as f32,
                ),
                Shape::Box {
                    width,
                    height,
                    depth,
                } => canvas.draw(Box).scale(
                    *width as f32,
                    *height as f32,
                    *depth as f32,
                ),
            }
            .translate(
                sb.position().x as f32,
                sb.position().y as f32,
                sb.position().z as f32,
            );
        }
    }
}
