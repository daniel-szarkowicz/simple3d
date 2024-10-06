use std::{cell::UnsafeCell, marker::PhantomData};

use crate::{
    rtree::{Leaf, RTree, AABBS},
    Float, Mat3, ObjID, Quat, Shape, Vec3, World,
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
    // These vectors MUST be sorted by id
    rtree: RTree<usize>,
    id: Vec<usize>,
    shape: Vec<UnsafeCell<Shape>>,
    inv_mass: Vec<UnsafeCell<Float>>,
    inv_inertia: Vec<UnsafeCell<Mat3>>,
    position: Vec<UnsafeCell<Vec3>>,
    rotation: Vec<UnsafeCell<Quat>>,
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

        let id = self.id_counter;
        self.id_counter += 1;
        self.id.push(id);
        self.shape.push(UnsafeCell::new(shape));
        self.inv_mass.push(UnsafeCell::new(inv_mass));
        self.inv_inertia.push(UnsafeCell::new(inv_inertia));
        self.position.push(UnsafeCell::new(position));
        self.rotation.push(UnsafeCell::new(rotation));
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

    pub(crate) fn aabbs(&self) -> AABBS<usize> {
        self.rtree.aabbs()
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
    staticbodies: &'rbs Rigidbodies,
}

impl<'rbs> RigidbodyRef<'rbs> {
    /// # SAFETY
    /// Callers must guarantee, that index is in bounds.
    /// Callers must guarantee, that there are no mutable references.
    pub(crate) unsafe fn new(
        index: usize,
        staticbodies: &'rbs Rigidbodies,
    ) -> Self {
        Self {
            index,
            staticbodies,
        }
    }

    pub fn shape(&self) -> &Shape {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell = unsafe { self.staticbodies.shape.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no mutable references.
        unsafe { &*cell.get() }
    }

    pub fn mass(&self) -> Float {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.staticbodies.inv_mass.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no mutable references.
        let inv_mass = unsafe { &*cell.get() };
        inv_mass.recip()
    }

    pub fn position(&self) -> &Vec3 {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.staticbodies.position.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no mutable references.
        unsafe { &*cell.get() }
    }

    pub fn rotation(&self) -> &Quat {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.staticbodies.rotation.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no mutable references.
        unsafe { &*cell.get() }
    }
}

pub struct RigidbodyRefMut<'rbs> {
    index: usize,
    staticbodies: &'rbs Rigidbodies,
    _marker: PhantomData<&'rbs mut Rigidbodies>,
}

impl<'rbs> RigidbodyRefMut<'rbs> {
    /// # SAFETY
    /// Callers must guarantee, that index is in bounds.
    /// Callers must guarantee, that there are no other references.
    pub(crate) unsafe fn new(
        index: usize,
        staticbodies: &'rbs Rigidbodies,
    ) -> Self {
        Self {
            index,
            staticbodies,
            _marker: PhantomData,
        }
    }

    pub fn shape(&mut self) -> &mut Shape {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell = unsafe { self.staticbodies.shape.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no other references.
        unsafe { &mut *cell.get() }
    }

    pub fn position(&mut self) -> &mut Vec3 {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.staticbodies.position.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no other references.
        unsafe { &mut *cell.get() }
    }

    pub fn rotation(&mut self) -> &mut Quat {
        // SAFETY: Caller guaranteed, that the index is in bounds.
        let cell =
            unsafe { self.staticbodies.rotation.get_unchecked(self.index) };
        // SAFETY: Caller guaranteed, that there are no other references.
        unsafe { &mut *cell.get() }
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
