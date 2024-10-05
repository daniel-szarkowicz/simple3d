use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::*;
use graphics::math::Transform;
use nalgebra::Vector3;
use physics::{ObjID, RigidbodyId, Shape, World};

fn main() {
    App::run_with(State::new());
}

struct State {
    world: World,
    sphere: RigidbodyId,
}

impl State {
    fn new() -> Self {
        let mut world = World::new();
        world
            .add_staticbody(Shape::Box {
                width: 10.0,
                height: 0.1,
                depth: 10.0,
            })
            .position(Vector3::new(0.0, -10.0, 0.0))
            .finish();
        let sphere = world
            .add_rigidbody(Shape::Sphere { diameter: 4.0 })
            .finish();
        Self { world, sphere }
    }
}

impl AppState for State {
    fn update(&mut self) {
        if let Some(mut sphere) = self.world.get_mut(self.sphere) {
            sphere.position().y -= 0.1 / 60.0;
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
        for rb in self.world.rigidbodies() {
            match rb.shape() {
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
                rb.position().x as f32,
                rb.position().y as f32,
                rb.position().z as f32,
            );
        }
    }
}
