use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::*;
use graphics::math::Transform;
use nalgebra::{Translation, Vector3};
use physics::{RigidbodyId, Shape, World};

fn main() {
    App::run_with(State::new());
}

struct State {
    world: World,
    test_body: RigidbodyId,
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
        let test_body = world
            .add_rigidbody(Shape::Box {
                width: 0.5,
                height: 0.5,
                depth: 0.5,
            })
            .position(Vector3::new(0.0, 10.0, 0.0))
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
        let mut body = world.get_mut(test_body).unwrap();
        body.apply_impulse(
            Vector3::new(0.1, 10.0, 0.1),
            Vector3::new(0.0, -0.01, 0.0),
        );
        Self { world, test_body }
    }
}

impl AppState for State {
    fn update(&mut self) {
        self.world.update(1.0 / 60.0);
    }

    fn draw(&self, canvas: &mut Canvas) {
        // for (aabb, _) in self.world.aabbs() {
        //     let pos = aabb.pos().cast();
        //     let size = aabb.size().cast();
        //     canvas
        //         .draw(BoxLines)
        //         .scale(size.x, size.y, size.z)
        //         .translate(pos.x, pos.y, pos.z);
        // }
        for rb in self.world.rigidbodies() {
            let transform = Translation::from(*rb.position()).to_homogeneous()
                * rb.rotation().to_rotation_matrix().to_homogeneous();
            let drawing = match rb.shape() {
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
            };
            drawing.transform(&transform.cast());
        }
    }
}
