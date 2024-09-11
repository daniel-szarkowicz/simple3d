use std::{any::TypeId, collections::HashMap, rc::Rc};

use crate::context::{Context, MeshId};

pub struct MeshManager {
    meshes: HashMap<TypeId, MeshId>,
    context: Rc<Context>,
}

impl MeshManager {
    pub fn new(context: Rc<Context>) -> Self {
        Self {
            meshes: HashMap::new(),
            context,
        }
    }

    // pub fn get_or_insert<T: MeshProvider>(&mut self, _: T) -> MeshId {
    //     *self
    //         .meshes
    //         .entry(TypeId::of::<T>())
    //         .or_insert_with(|| self.context.load_mesh(T::create_mesh()))
    // }
}

pub struct Mesh {
    pub name: String,
}

pub trait MeshProvider: 'static + Copy {
    fn create_mesh() -> Mesh;
}
