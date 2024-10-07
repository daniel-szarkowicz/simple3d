use std::{cell::UnsafeCell, marker::PhantomData};

use crate::{
    gjk::{self},
    rtree::{Leaf, QueryItem, RTree, AABBS},
    Float, Mat3, ObjID, Quat, Shape, Staticbodies, Vec3, World,
};

const DEFAULT_DENSITY: Float = 1.0;

pub struct RigidbodyBuilder<'w> {
    world: &'w mut World,
    shape: Shape,
    mass: Float,
    position: Vec3,
    rotation: Quat,
}

impl<'w> RigidbodyBuilder<'w> {
    pub fn new(world: &'w mut World, shape: Shape) -> Self {
        Self {
            world,
            shape,
            mass: shape.volume() * DEFAULT_DENSITY,
            position: Vec3::zeros(),
            rotation: Quat::identity(),
        }
    }

    pub fn mass(mut self, mass: Float) -> Self {
        self.mass = mass;
        self
    }

    pub fn position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn finish(self) -> RigidbodyId {
        self.world.rigidbodies.insert(
            self.shape,
            self.mass,
            self.position,
            self.rotation,
        )
    }
}

#[derive(Default)]
pub(crate) struct Rigidbodies {
    id_counter: usize,
    rtree: RTree<usize>,
    rb_contacts: Vec<(usize, Vec3, usize, Vec3, Vec3)>,
    sb_contacts: Vec<(usize, Vec3, usize, Vec3, Vec3)>,
    // These vectors MUST be sorted by id
    id: Vec<usize>,
    shape: Vec<UnsafeCell<Shape>>,
    inv_mass: Vec<UnsafeCell<Float>>,
    inv_inertia: Vec<UnsafeCell<Mat3>>,
    position: Vec<UnsafeCell<Vec3>>,
    rotation: Vec<UnsafeCell<Quat>>,
    momentum: Vec<UnsafeCell<Vec3>>,
    angular_momentum: Vec<UnsafeCell<Vec3>>,
}

impl Rigidbodies {
    fn insert(
        &mut self,
        shape: Shape,
        mass: Float,
        position: Vec3,
        rotation: Quat,
    ) -> RigidbodyId {
        let inv_mass = mass.recip();
        let inv_inertia = shape.inverse_inertia(mass);
        let momentum = Vec3::zeros();
        let angular_momentum = Vec3::zeros();

        let id = self.id_counter;
        self.id_counter += 1;
        self.id.push(id);
        self.shape.push(UnsafeCell::new(shape));
        self.inv_mass.push(UnsafeCell::new(inv_mass));
        self.inv_inertia.push(UnsafeCell::new(inv_inertia));
        self.position.push(UnsafeCell::new(position));
        self.rotation.push(UnsafeCell::new(rotation));
        self.momentum.push(UnsafeCell::new(momentum));
        self.angular_momentum
            .push(UnsafeCell::new(angular_momentum));
        RigidbodyId(id)
    }

    fn get(&self, id: RigidbodyId) -> Option<RigidbodyRef> {
        let index = self.id.binary_search(&id.0).ok()?;
        // SAFETY
        // index returned by binary_search is in bounds
        // we have a reference to self, no mutable references can exist
        Some(unsafe { RigidbodyRef::new(index, self) })
    }

    fn get_mut(&mut self, id: RigidbodyId) -> Option<RigidbodyRefMut> {
        let index = self.id.binary_search(&id.0).ok()?;
        // SAFETY
        // index returned by binary_search is in bounds
        // we have a mutable refernce to self, no other references can exist
        Some(unsafe { RigidbodyRefMut::new(index, self) })
    }

    fn remove(&mut self, id: RigidbodyId) {
        let Ok(index) = self.id.binary_search(&id.0) else {
            return;
        };
        self.id.remove(index);
        self.shape.remove(index);
        self.inv_mass.remove(index);
        self.inv_inertia.remove(index);
        self.position.remove(index);
        self.rotation.remove(index);
        self.momentum.remove(index);
        self.angular_momentum.remove(index);
    }

    // TODO: properly update the tree instead of creating a new one
    pub(crate) fn update_rtree(&mut self) {
        let leaves = (0..self.id.len())
            .map(|i| Leaf {
                aabb: crate::aabb(
                    self.shape[i].get_mut(),
                    self.position[i].get_mut(),
                    self.rotation[i].get_mut(),
                ),
                data: i,
            })
            .collect();
        self.rtree = RTree::new(leaves);
    }

    pub(crate) fn update_bodies(&mut self, delta: Float) {
        for i in 0..self.id.len() {
            *self.position[i].get_mut() += delta
                * *self.inv_mass[i].get_mut()
                * *self.momentum[i].get_mut();
            let rotation_matrix =
                self.rotation[i].get_mut().to_rotation_matrix();
            let inverse_inertia = rotation_matrix
                * *self.inv_inertia[i].get_mut()
                * rotation_matrix.inverse();
            *self.rotation[i].get_mut() *= Quat::new(
                delta * inverse_inertia * *self.angular_momentum[i].get_mut(),
            );
        }
    }

    pub(crate) fn update_rb_contacts(&mut self) {
        self.rb_contacts.clear();
        self.rb_contacts.extend(
            self.rtree
                .leaves()
                .flat_map(|Leaf { aabb, data: i }| {
                    self.rtree
                        .query(*aabb)
                        .filter_map(|QueryItem { aabb: _, data }| match data {
                            crate::rtree::QueryData::Node { depth: _ } => None,
                            crate::rtree::QueryData::Leaf { data } => {
                                Some(*data)
                            }
                        })
                        .map(move |j| (*i, j))
                })
                .filter(|(i, j)| i < j)
                .filter_map(|(i, j)| {
                    let s1 = unsafe { &*self.shape[i].get() };
                    let s2 = unsafe { &*self.shape[j].get() };
                    let p1 = unsafe { &*self.position[i].get() };
                    let p2 = unsafe { &*self.position[j].get() };
                    let r1 = unsafe { &*self.rotation[i].get() };
                    let r2 = unsafe { &*self.rotation[j].get() };
                    check_contact(s1, p1, r1, s2, p2, r2)
                        .map(|(v1, v2, n)| (i, v1, j, v2, n))
                }),
        );
    }

    pub(crate) fn update_sb_contacts(&mut self, sbs: &Staticbodies) {
        self.sb_contacts.clear();
        self.sb_contacts.extend(
            self.rtree
                .leaves()
                .flat_map(|Leaf { aabb, data: i }| {
                    sbs.rtree
                        .query(*aabb)
                        .filter_map(|QueryItem { aabb: _, data }| match data {
                            crate::rtree::QueryData::Node { depth: _ } => None,
                            crate::rtree::QueryData::Leaf { data } => {
                                Some(*data)
                            }
                        })
                        .map(move |j| (*i, j))
                })
                .filter(|(i, j)| i < j)
                .filter_map(|(i, j)| {
                    let s1 = unsafe { &*self.shape[i].get() };
                    let s2 = unsafe { &*sbs.shape[j].get() };
                    let p1 = unsafe { &*self.position[i].get() };
                    let p2 = unsafe { &*sbs.position[j].get() };
                    let r1 = unsafe { &*self.rotation[i].get() };
                    let r2 = unsafe { &*sbs.rotation[j].get() };
                    check_contact(s1, p1, r1, s2, p2, r2)
                        .map(|(v1, v2, n)| (i, v1, j, v2, n))
                }),
        );
    }

    pub(crate) fn aabbs(&self) -> AABBS<usize> {
        self.rtree.aabbs()
    }

    pub(crate) fn contacts(
        &self,
    ) -> impl Iterator<Item = (Vec3, Vec3, Vec3)> + '_ {
        self.rb_contacts
            .iter()
            .chain(self.sb_contacts.iter())
            .map(|(_, v1, _, v2, n)| (*v1, *v2, *n))
    }
}

fn check_contact(
    s1: &Shape,
    p1: &Vec3,
    r1: &Quat,
    s2: &Shape,
    p2: &Vec3,
    r2: &Quat,
) -> Option<(Vec3, Vec3, Vec3)> {
    gjk::gjk(&(s1, p1, r1), &(s2, p2, r2))
}

impl gjk::Support for (&Shape, &Vec3, &Quat) {
    fn support(&self, direction: &Vec3) -> Vec3 {
        match self.0 {
            Shape::Sphere { diameter: _ } => *self.1,
            Shape::Box {
                width,
                height,
                depth,
            } => {
                let model_dir = self.2.inverse_transform_vector(direction);
                let model_pos = 0.5 * model_dir.map(Float::signum);
                self.2.transform_vector(
                    &model_pos
                        .component_mul(&Vec3::new(*width, *height, *depth)),
                ) + self.1
            }
        }
    }

    fn radius(&self) -> f64 {
        match self.0 {
            Shape::Sphere { diameter } => diameter / 2.0,
            Shape::Box {
                width: _,
                height: _,
                depth: _,
            } => 0.0,
        }
    }

    fn base(&self) -> Vec3 {
        *self.1
    }
}

#[derive(Clone, Copy)]
pub struct RigidbodyId(usize);

impl<'w> ObjID<'w> for RigidbodyId {
    type Ref = RigidbodyRef<'w>;

    type RefMut = RigidbodyRefMut<'w>;

    fn get(self, world: &'w World) -> Option<Self::Ref> {
        world.rigidbodies.get(self)
    }

    fn get_mut(self, world: &'w mut World) -> Option<Self::RefMut> {
        world.rigidbodies.get_mut(self)
    }

    fn remove(self, world: &'w mut World) {
        world.rigidbodies.remove(self)
    }
}

pub struct RigidbodyRef<'rbs> {
    index: usize,
    rigidbodies: &'rbs Rigidbodies,
}

impl<'rbs> RigidbodyRef<'rbs> {
    /// # SAFETY
    /// Callers must guarantee, that index is in bounds.
    /// Callers must guarantee, that there are no mutable references.
    pub(crate) unsafe fn new(
        index: usize,
        rigidbodies: &'rbs Rigidbodies,
    ) -> Self {
        Self { index, rigidbodies }
    }

    pub fn shape(&self) -> &Shape {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell = unsafe { self.rigidbodies.shape.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no mutable references.
        unsafe { &*cell.get() }
    }

    pub fn mass(&self) -> Float {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.rigidbodies.inv_mass.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no mutable references.
        let inv_mass = unsafe { &*cell.get() };
        inv_mass.recip()
    }

    pub fn position(&self) -> &Vec3 {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.rigidbodies.position.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no mutable references.
        unsafe { &*cell.get() }
    }

    pub fn rotation(&self) -> &Quat {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.rigidbodies.rotation.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no mutable references.
        unsafe { &*cell.get() }
    }
}

pub struct RigidbodyRefMut<'rbs> {
    index: usize,
    rigidbodies: &'rbs Rigidbodies,
    _marker: PhantomData<&'rbs mut Rigidbodies>,
}

impl<'rbs> RigidbodyRefMut<'rbs> {
    /// # SAFETY
    /// Callers must guarantee, that index is in bounds.
    /// Callers must guarantee, that there are no other references.
    pub(crate) unsafe fn new(
        index: usize,
        rigidbodies: &'rbs Rigidbodies,
    ) -> Self {
        Self {
            index,
            rigidbodies,
            _marker: PhantomData,
        }
    }

    pub fn shape(&mut self) -> &mut Shape {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell = unsafe { self.rigidbodies.shape.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no other references.
        unsafe { &mut *cell.get() }
    }

    pub fn position(&mut self) -> &mut Vec3 {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.rigidbodies.position.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no other references.
        unsafe { &mut *cell.get() }
    }

    pub fn rotation(&mut self) -> &mut Quat {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.rigidbodies.rotation.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no other references.
        unsafe { &mut *cell.get() }
    }

    pub fn momentum(&mut self) -> &mut Vec3 {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.rigidbodies.momentum.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no other references.
        unsafe { &mut *cell.get() }
    }

    pub fn angular_momentum(&mut self) -> &mut Vec3 {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell = unsafe {
            self.rigidbodies.angular_momentum.get_unchecked(self.index)
        };
        // SAFETY: Caller guaranteed, that there are no other references.
        unsafe { &mut *cell.get() }
    }

    pub fn apply_impulse(&mut self, attack_point: Vec3, impulse: Vec3) {
        *self.momentum() += impulse;
        let offset = attack_point - *self.position();
        *self.angular_momentum() += offset.cross(&impulse);
    }
}

pub struct RigidbodyIter<'rbs> {
    index: usize,
    rigidbodies: &'rbs Rigidbodies,
}

impl<'rbs> RigidbodyIter<'rbs> {
    pub(crate) fn new(rigidbodies: &'rbs Rigidbodies) -> Self {
        Self {
            index: 0,
            rigidbodies,
        }
    }
}

impl<'rbs> Iterator for RigidbodyIter<'rbs> {
    type Item = RigidbodyRef<'rbs>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.rigidbodies.id.len() {
            // SAFETY
            // index was bounds checked above.
            // We have a reference to rigidbodies,
            // no mutable reference can exist.
            let result =
                unsafe { RigidbodyRef::new(self.index, self.rigidbodies) };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

pub struct RigidbodyIterMut<'rbs> {
    index: usize,
    rigidbodies: &'rbs Rigidbodies,
    _marker: PhantomData<&'rbs mut Rigidbodies>,
}

impl<'rbs> RigidbodyIterMut<'rbs> {
    pub(crate) fn new(rigidbodies: &'rbs mut Rigidbodies) -> Self {
        Self {
            index: 0,
            rigidbodies,
            _marker: PhantomData,
        }
    }
}

impl<'rbs> Iterator for RigidbodyIterMut<'rbs> {
    type Item = RigidbodyRefMut<'rbs>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.rigidbodies.id.len() {
            // SAFETY
            // index was bounds checked above.
            // This iterator was created from a mutable reference to
            // rigidbodies, therefore no other references can exist to it.
            let result =
                unsafe { RigidbodyRefMut::new(self.index, self.rigidbodies) };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}
