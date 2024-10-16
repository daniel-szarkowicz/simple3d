use graphics::{
    app::{App, AppState},
    geometry::{Box, Ellipsoid},
    math::Transform,
};
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
        for x in -7..=7 {
            for y in -7..=7 {
                for z in 0..5 {
                    world
                        .add_rigidbody(Shape::Box {
                            width: 1.0,
                            height: 1.0,
                            depth: 1.0,
                        })
                        .position(Vector3::new(
                            f64::from(x) * 1.01,
                            f64::from(y) * 1.01,
                            f64::from(z) * 1.01,
                        ))
                        .finish();
                }
            }
        }
        let bigcube = world
            .add_rigidbody(Shape::Box {
                width: 4.0,
                height: 4.0,
                depth: 4.0,
            })
            .position(Vector3::new(0.0, 0.0, -100.0))
            .finish();
        let mut bigcube = world.get_mut(bigcube).unwrap();
        bigcube.apply_central_impulse(Vector3::new(0.0, 0.0, 1000.0));
        Self { world }
    }
}

impl AppState for State {
    fn update(&mut self) {
        self.world.update(1.0 / 60.0);
    }

    fn draw(&self, canvas: &mut graphics::canvas::Canvas) {
        canvas
            .group(|canvas| {
                for rb in self.world.rigidbodies() {
                    let transform = Translation::from(*rb.position())
                        .to_homogeneous()
                        * rb.rotation().to_rotation_matrix().to_homogeneous();
                    let drawing = match rb.shape() {
                        Shape::Sphere { diameter } => {
                            canvas.draw(Ellipsoid).scale(
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
                    drawing.transform(&transform.cast());
                }

                for (p1, p2, _) in self.world.contacts() {
                    canvas
                        .draw(Ellipsoid)
                        .color([1.0, 0.0, 0.0])
                        .scale(0.1, 0.1, 0.1)
                        .translate(p1.x as f32, p1.y as f32, p1.z as f32);
                    canvas
                        .draw(Ellipsoid)
                        .color([1.0, 0.0, 0.0])
                        .scale(0.1, 0.1, 0.1)
                        .translate(p2.x as f32, p2.y as f32, p2.z as f32);
                }
            })
            .translate_x(10.0);
    }
}
