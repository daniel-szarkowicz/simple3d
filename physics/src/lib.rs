mod gjk;
mod staticbody;
pub use staticbody::*;
mod rigidbody;
pub use rigidbody::*;
pub mod rtree;
use rtree::AABB;
mod world;
pub use world::*;

pub(crate) type Float = f64;
pub(crate) type Vec3 = nalgebra::Vector3<Float>;
pub(crate) type Quat = nalgebra::UnitQuaternion<Float>;
pub(crate) type Mat3 = nalgebra::Matrix3<Float>;
pub(crate) type Mat4 = nalgebra::Matrix4<Float>;

#[derive(Clone, Copy)]
pub enum Shape {
    Sphere {
        diameter: Float,
    },
    Box {
        width: Float,
        height: Float,
        depth: Float,
    },
}

impl Shape {
    pub fn volume(&self) -> Float {
        match self {
            Shape::Sphere { diameter } => {
                std::f64::consts::PI / 6.0 * diameter * diameter * diameter
            }
            Shape::Box {
                width,
                height,
                depth,
            } => width * height * depth,
        }
    }

    pub fn inverse_inertia(&self, mass: Float) -> Mat3 {
        let inertia = match self {
            Self::Sphere { diameter } => {
                Mat3::identity() / 6.0 * mass * *diameter * *diameter
            }
            #[rustfmt::skip]
            Shape::Box {
                width,
                height,
                depth,
            } => Mat3::new(
                mass / 12.0 * (height * height + depth * depth), 0.0, 0.0,
                0.0, mass / 12.0 * (width * width + depth * depth), 0.0,
                0.0, 0.0, mass / 12.0 * (width * width+ height * height),
            ),
        };
        inertia.try_inverse().expect("Inertia tensor is invertible")
    }
}

pub(crate) fn aabb(shape: &Shape, position: &Vec3, rotation: &Quat) -> AABB {
    match shape {
        Shape::Sphere { diameter } => {
            let r = diameter / 2.0;
            let offset = Vec3::new(r, r, r);
            AABB {
                min: position - offset,
                max: position + offset,
            }
        }
        Shape::Box {
            width,
            height,
            depth,
        } => {
            let rotation = rotation.to_rotation_matrix();
            let mut min = *position;
            let mut max = *position;
            for x in [-0.5, 0.5] {
                for y in [-0.5, 0.5] {
                    for z in [-0.5, 0.5] {
                        let p = position
                            + rotation
                                * Vec3::new(
                                    x * (width + 0.01),
                                    y * (height + 0.01),
                                    z * (depth + 0.01),
                                );
                        min = min.inf(&p);
                        max = max.sup(&p);
                    }
                }
            }
            AABB { min, max }
        }
    }
}

pub trait ObjID<'w>: Copy {
    type Ref: 'w;
    type RefMut: 'w;

    fn get(self, world: &'w World) -> Option<Self::Ref>;
    fn get_mut(self, world: &'w mut World) -> Option<Self::RefMut>;

    fn remove(self, world: &'w mut World);
}
