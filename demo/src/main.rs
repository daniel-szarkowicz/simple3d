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
                width: 10.0,
                height: 0.1,
                depth: 10.0,
            })
            .position(Vector3::new(0.0, -10.0, 0.0))
            .finish();
        for x in -5..=5 {
            for y in -5..=5 {
                for z in -5..=5 {
                    world
                        .add_rigidbody(Shape::Sphere { diameter: 0.5 })
                        .position(Vector3::new(x as f64, y as f64, z as f64))
                        .finish();
                }
            }
        }
        world.update();
        Self { world }
    }
}

impl AppState for State {
    fn update(&mut self) {}

    fn draw(&self, canvas: &mut Canvas) {
        for (aabb, _) in self.world.aabbs() {
            let pos = aabb.pos().cast();
            let size = aabb.size().cast();
            canvas
                .draw(BoxLines)
                .scale(size.x, size.y, size.z)
                .translate(pos.x, pos.y, pos.z);
        }
    }
}
