mod rigidbody;
mod staticbody;
pub use rigidbody::*;
pub use staticbody::*;

pub(crate) type Float = f64;
pub(crate) type Vec3 = nalgebra::Vector3<Float>;
pub(crate) type Quat = nalgebra::UnitQuaternion<Float>;
pub(crate) type Mat3 = nalgebra::Matrix3<Float>;
pub(crate) type Mat4 = nalgebra::Matrix4<Float>;

#[derive(Default)]
pub struct World {
    staticbodies: Staticbodies,
    rigidbodies: Rigidbodies,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<'w, ID: ObjID<'w>>(&'w self, id: ID) -> Option<ID::Ref> {
        id.get(self)
    }

    pub fn get_mut<'w, ID: ObjID<'w>>(
        &'w mut self,
        id: ID,
    ) -> Option<ID::RefMut> {
        id.get_mut(self)
    }

    pub fn add_staticbody(&mut self, shape: Shape) -> StaticbodyBuilder {
        StaticbodyBuilder::new(self, shape)
    }

    pub fn staticbodies(&self) -> StaticbodyIter {
        StaticbodyIter::new(&self.staticbodies)
    }

    pub fn staticbodies_mut(&mut self) -> StaticbodyIterMut {
        StaticbodyIterMut::new(&mut self.staticbodies)
    }

    pub fn add_rigidbody(&mut self, shape: Shape) -> RigidbodyBuilder {
        RigidbodyBuilder::new(self, shape)
    }

    pub fn rigidbodies(&self) -> RigidbodyIter {
        RigidbodyIter::new(&self.rigidbodies)
    }

    pub fn rigidbodies_mut(&mut self) -> RigidbodyIterMut {
        RigidbodyIterMut::new(&mut self.rigidbodies)
    }
}

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

pub trait ObjID<'w>: Copy {
    type Ref: 'w;
    type RefMut: 'w;

    fn get(self, world: &'w World) -> Option<Self::Ref>;
    fn get_mut(self, world: &'w mut World) -> Option<Self::RefMut>;

    fn remove(self, world: &'w mut World);
}
