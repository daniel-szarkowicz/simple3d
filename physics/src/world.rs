use crate::{
    rtree::AABB, Float, ObjID, Rigidbodies, RigidbodyBuilder, RigidbodyIter,
    RigidbodyIterMut, Shape, Staticbodies, StaticbodyBuilder, StaticbodyIter,
    StaticbodyIterMut, Vec3,
};

#[derive(Default)]
pub struct World {
    pub(crate) staticbodies: Staticbodies,
    pub(crate) rigidbodies: Rigidbodies,
}

const GRAVITY: Vec3 = Vec3::new(0.0, -1.0, 0.0);
// const GRAVITY: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const SOLVER_STEPS: usize = 10;

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

    pub fn contacts(&self) -> impl Iterator<Item = (Vec3, Vec3, Vec3)> + '_ {
        self.rigidbodies.contacts()
    }

    pub fn update(&mut self, delta: Float) {
        self.rigidbodies.update_bodies(delta);
        for mut rb in self.rigidbodies_mut() {
            if *rb.inv_mass() != 0.0 {
                // rb.apply_central_force(GRAVITY * rb.mass());
                rb.apply_central_impulse(GRAVITY * rb.mass() * delta);
            }
        }
        self.staticbodies.update_rtree();
        self.rigidbodies.update_rtree();
        self.rigidbodies.update_rb_contacts();
        self.rigidbodies.update_sb_contacts(&self.staticbodies);
        for _ in 0..SOLVER_STEPS {
            self.rigidbodies.resolve_rb_contacts();
            self.rigidbodies.resolve_sb_contacts(&self.staticbodies);
        }
    }

    pub fn aabbs(&self) -> impl Iterator<Item = (&AABB, usize)> {
        self.staticbodies.aabbs().chain(self.rigidbodies.aabbs())
    }
}
