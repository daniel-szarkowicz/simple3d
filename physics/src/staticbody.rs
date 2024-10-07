use std::{cell::UnsafeCell, iter::zip, marker::PhantomData};

use crate::{
    rtree::{Leaf, RTree, AABBS},
    ObjID, Quat, Shape, Vec3, World,
};

#[must_use]
pub struct StaticbodyBuilder<'w> {
    world: &'w mut World,
    shape: Shape,
    position: Vec3,
    rotation: Quat,
}

impl<'w> StaticbodyBuilder<'w> {
    pub fn new(world: &'w mut World, shape: Shape) -> Self {
        Self {
            world,
            shape,
            position: Vec3::zeros(),
            rotation: Quat::identity(),
        }
    }

    pub fn position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn finish(self) -> StaticbodyId {
        self.world
            .staticbodies
            .insert(self.shape, self.position, self.rotation)
    }
}

#[derive(Default)]
pub(crate) struct Staticbodies {
    id_counter: usize,
    // These vectors MUST be sorted by id
    pub(crate) rtree: RTree<usize>,
    id: Vec<usize>,
    pub(crate) shape: Vec<UnsafeCell<Shape>>,
    pub(crate) position: Vec<UnsafeCell<Vec3>>,
    pub(crate) rotation: Vec<UnsafeCell<Quat>>,
}

impl Staticbodies {
    fn insert(
        &mut self,
        shape: Shape,
        position: Vec3,
        rotation: Quat,
    ) -> StaticbodyId {
        let id = self.id_counter;
        self.id_counter += 1;
        self.id.push(id);
        self.shape.push(UnsafeCell::new(shape));
        self.position.push(UnsafeCell::new(position));
        self.rotation.push(UnsafeCell::new(rotation));
        StaticbodyId(id)
    }

    fn get(&self, id: StaticbodyId) -> Option<StaticbodyRef> {
        let index = self.id.binary_search(&id.0).ok()?;
        // SAFETY
        // index returned by binary_search is in bounds
        // we have a reference to self, no mutable references can exist
        Some(unsafe { StaticbodyRef::new(index, self) })
    }

    fn get_mut(&mut self, id: StaticbodyId) -> Option<StaticbodyRefMut> {
        let index = self.id.binary_search(&id.0).ok()?;
        // SAFETY
        // index returned by binary_search is in bounds
        // we have a mutable refernce to self, no other references can exist
        Some(unsafe { StaticbodyRefMut::new(index, self) })
    }

    fn remove(&mut self, id: StaticbodyId) {
        let Ok(index) = self.id.binary_search(&id.0) else {
            return;
        };
        self.id.remove(index);
        self.shape.remove(index);
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

pub struct StaticbodyIter<'sbs> {
    index: usize,
    staticbodies: &'sbs Staticbodies,
}

impl<'sbs> StaticbodyIter<'sbs> {
    pub(crate) fn new(staticbodies: &'sbs Staticbodies) -> Self {
        Self {
            index: 0,
            staticbodies,
        }
    }
}

impl<'sbs> Iterator for StaticbodyIter<'sbs> {
    type Item = StaticbodyRef<'sbs>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.staticbodies.id.len() {
            // SAFETY
            // index was bounds checked above.
            // We have a reference to staticbodies,
            // no mutable reference can exist.
            let result =
                unsafe { StaticbodyRef::new(self.index, self.staticbodies) };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

pub struct StaticbodyIterMut<'sbs> {
    index: usize,
    staticbodies: &'sbs Staticbodies,
    _marker: PhantomData<&'sbs mut Staticbodies>,
}

impl<'sbs> StaticbodyIterMut<'sbs> {
    pub(crate) fn new(staticbodies: &'sbs mut Staticbodies) -> Self {
        Self {
            index: 0,
            staticbodies,
            _marker: PhantomData,
        }
    }
}

impl<'sbs> Iterator for StaticbodyIterMut<'sbs> {
    type Item = StaticbodyRefMut<'sbs>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.staticbodies.id.len() {
            // SAFETY
            // index was bounds checked above.
            // This iterator was created from a mutable reference to
            // staticbodies, therefore no other references can exist to it.
            let result =
                unsafe { StaticbodyRefMut::new(self.index, self.staticbodies) };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub struct StaticbodyId(usize);

impl<'w> ObjID<'w> for StaticbodyId {
    type Ref = StaticbodyRef<'w>;

    type RefMut = StaticbodyRefMut<'w>;

    fn get(self, world: &'w World) -> Option<Self::Ref> {
        world.staticbodies.get(self)
    }

    fn get_mut(self, world: &'w mut World) -> Option<Self::RefMut> {
        world.staticbodies.get_mut(self)
    }

    fn remove(self, world: &'w mut World) {
        world.staticbodies.remove(self)
    }
}

pub struct StaticbodyRef<'sbs> {
    index: usize,
    staticbodies: &'sbs Staticbodies,
}

impl<'sbs> StaticbodyRef<'sbs> {
    /// # SAFETY
    /// Callers must guarantee, that index is in bounds.
    /// Callers must guarantee, that there are no mutable references.
    pub(crate) unsafe fn new(
        index: usize,
        staticbodies: &'sbs Staticbodies,
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

pub struct StaticbodyRefMut<'sbs> {
    index: usize,
    staticbodies: &'sbs Staticbodies,
    _marker: PhantomData<&'sbs mut Staticbodies>,
}

impl<'sbs> StaticbodyRefMut<'sbs> {
    /// # SAFETY
    /// Callers must guarantee, that index is in bounds.
    /// Callers must guarantee, that there are no other references.
    pub(crate) unsafe fn new(
        index: usize,
        staticbodies: &'sbs Staticbodies,
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
