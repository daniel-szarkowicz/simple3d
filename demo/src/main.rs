use graphics::app::{App, AppState};
use graphics::canvas::Canvas;
use graphics::geometry::*;
use graphics::math::Transform;
use nalgebra::{Translation, UnitQuaternion, Vector3};
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
                width: 100.0,
                height: 10.0,
                depth: 100.0,
            })
            .position(Vector3::new(0.0, -15.0, 0.0))
            .finish();
        let test_body = world
            .add_rigidbody(Shape::Box {
                width: 0.5,
                height: 0.5,
                depth: 0.5,
            })
            .position(Vector3::new(0.0, 10.0, 0.0))
            .finish();
        for x in -0..=0 {
            for y in -3..=3 {
                for z in -0..=0 {
                    let xyz = Vector3::new(x as f64, y as f64, z as f64);
                    world
                        // .add_rigidbody(Shape::Sphere { diameter: 0.5 })
                        .add_rigidbody(Shape::Box {
                            width: 0.5,
                            height: 0.5,
                            depth: 0.5,
                        })
                        .position(xyz)
                        .rotation(UnitQuaternion::new(xyz))
                        .finish();
                }
            }
        }
        let mut body = world.get_mut(test_body).unwrap();
        body.apply_impulse(
            Vector3::new(0.1, 10.0, 0.1),
            Vector3::new(0.0, -0.5, 0.0),
        );
        Self { world, test_body }
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
                Shape::Sphere { diameter } => {
                    canvas.draw(EllipsoidLines).scale(
                        *diameter as f32,
                        *diameter as f32,
                        *diameter as f32,
                    )
                }
                Shape::Box {
                    width,
                    height,
                    depth,
                } => canvas.draw(BoxLines).scale(
                    *width as f32,
                    *height as f32,
                    *depth as f32,
                ),
            };
            drawing.transform(&transform.cast());
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
