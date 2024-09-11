use std::{any::TypeId, collections::HashMap, ops::Range, sync::Arc};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

pub struct MeshManager {
    meshes: HashMap<TypeId, Arc<MeshBuffers>>,
    device: Arc<Device>,
}

impl MeshManager {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            meshes: HashMap::new(),
            device,
        }
    }

    pub fn get_or_insert<T: MeshProvider>(&mut self, _: T) -> Arc<MeshBuffers> {
        self.meshes
            .entry(TypeId::of::<T>())
            .or_insert_with(|| load_mesh(&self.device, T::create_mesh()))
            .clone()
    }
}

fn load_mesh(device: &Device, mesh: Mesh) -> Arc<MeshBuffers> {
    let index_range = 0..mesh.indices.len() as u32;
    let vertex = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&mesh.vertices),
        usage: BufferUsages::VERTEX,
    });
    let index = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&mesh.indices),
        usage: BufferUsages::INDEX,
    });
    Arc::new(MeshBuffers {
        vertex,
        index,
        index_range,
    })
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub trait MeshProvider: 'static + Copy {
    fn create_mesh() -> Mesh;
}

pub struct MeshBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
    pub index_range: Range<u32>,
}
