use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::*;
use graphics::math::Transform;
use nalgebra::{Translation, Vector3};
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
                width: 1000.0,
                height: 9.0,
                depth: 1000.0,
            })
            .position(Vector3::new(0.0, -5.0, 0.0))
            .finish();
        // for i in 1..5 {
        //     world
        //         .add_rigidbody(Shape::Sphere { diameter: 1.0 })
        //         .position(Vector3::new(0.01 * i as f64, 5.0 * i as f64, 0.0))
        //         .finish();
        // }
        world
            .add_rigidbody(Shape::Box {
                width: 1.0,
                height: 1.0,
                depth: 1.0,
            })
            .mass(100.0)
            .position(Vector3::new(0.0, 5.0, 0.0))
            .finish();
        Self { world }
    }
}

impl AppState for State {
    fn update(&mut self) {
        self.world.update(1.0 / 60.0);
    }

    fn draw(&self, canvas: &mut Canvas) {
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
            drawing.transform(&transform.cast()).color([0.0, 1.0, 0.0]);
        }
        for sb in self.world.staticbodies() {
            let transform = Translation::from(*sb.position()).to_homogeneous()
                * sb.rotation().to_rotation_matrix().to_homogeneous();
            let drawing = match sb.shape() {
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
        for (v1, v2, _) in self.world.contacts() {
            canvas
                .draw(Ellipsoid)
                .scale(0.05, 0.05, 0.05)
                .translate(v1.x as f32, v1.y as f32, v1.z as f32)
                .color([1.0, 0.0, 0.0]);
            canvas
                .draw(Ellipsoid)
                .scale(0.05, 0.05, 0.05)
                .translate(v2.x as f32, v2.y as f32, v2.z as f32)
                .color([1.0, 0.0, 0.0]);
        }
    }
}
