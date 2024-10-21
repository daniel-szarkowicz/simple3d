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
        let thickness = 1.0;
        let dist = 5.0;
        let size = dist * 2.0 + thickness;
        let mut world = World::new();
        world
            .add_staticbody(Shape::Box {
                width: size,
                height: thickness,
                depth: size,
            })
            .position(Vector3::new(0.0, -dist, 0.0))
            .finish();
        world
            .add_staticbody(Shape::Box {
                width: thickness,
                height: size,
                depth: size,
            })
            .position(Vector3::new(dist, 0.0, 0.0))
            .finish();
        world
            .add_staticbody(Shape::Box {
                width: thickness,
                height: size,
                depth: size,
            })
            .position(Vector3::new(-dist, 0.0, 0.0))
            .finish();
        world
            .add_staticbody(Shape::Box {
                width: size,
                height: size,
                depth: thickness,
            })
            .position(Vector3::new(0.0, 0.0, dist))
            .finish();
        world
            .add_staticbody(Shape::Box {
                width: size,
                height: size,
                depth: thickness,
            })
            .position(Vector3::new(0.0, 0.0, -dist))
            .finish();
        for i in (1..=1000).map(f64::from) {
            world
                .add_rigidbody(Shape::Sphere { diameter: 1.0 })
                // .add_rigidbody(Shape::Box {
                //     width: 1.0,
                //     height: 1.0,
                //     depth: 1.0,
                // })
                .position(Vector3::new(
                    i.cos() * (i / 10.0).cos() * (dist - thickness),
                    i,
                    i.sin() * (i / 10.0).cos() * (dist - thickness),
                ))
                .finish();
        }
        // world
        //     // .add_rigidbody(Shape::Box {
        //     //     width: 1.0,
        //     //     height: 1.0,
        //     //     depth: 1.0,
        //     // })
        //     .add_rigidbody(Shape::Sphere { diameter: 1.0 })
        //     .position(Vector3::new(0.0, 0.5, 0.0))
        //     .mass(100.0)
        //     .finish();
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
                Shape::Sphere { diameter } => {
                    canvas.draw(StaticLowPoly(Ellipsoid)).scale(
                        *diameter as f32,
                        *diameter as f32,
                        *diameter as f32,
                    )
                }
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
                .color([0.0, 0.0, 1.0]);
        }
    }
}
