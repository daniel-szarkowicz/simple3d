mod staticbody;
pub use staticbody::*;

pub(crate) type Float = f64;
pub(crate) type Vec3 = nalgebra::Vector3<Float>;
pub(crate) type Quat = nalgebra::UnitQuaternion<Float>;
pub(crate) type Mat4 = nalgebra::Matrix4<Float>;

#[derive(Default)]
pub struct World {
    staticbodies: Staticbodies,
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

pub trait ObjID<'w> {
    type Ref: 'w;
    type RefMut: 'w;

    fn get(self, world: &'w World) -> Option<Self::Ref>;
    fn get_mut(self, world: &'w mut World) -> Option<Self::RefMut>;

    fn remove(self, world: &'w mut World);
}
