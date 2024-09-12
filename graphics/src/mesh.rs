use std::{
    any::TypeId, cmp::Ordering, collections::HashMap, ops::Range, sync::Arc,
};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, VertexAttribute, VertexBufferLayout,
};

pub struct MeshManager {
    mesh_ids: HashMap<TypeId, MeshId>,
    meshes: Vec<MeshBuffers>,
    device: Arc<Device>,
}

impl MeshManager {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            mesh_ids: HashMap::new(),
            meshes: Vec::new(),
            device,
        }
    }

    pub fn get_or_insert<T: MeshProvider>(&mut self, _: T) -> MeshId {
        *self.mesh_ids.entry(TypeId::of::<T>()).or_insert_with(|| {
            let id = self.meshes.len();
            self.meshes.push(load_mesh(&self.device, T::create_mesh()));
            MeshId(id)
        })
    }

    pub(crate) fn get_by_id(&self, id: MeshId) -> &MeshBuffers {
        &self.meshes[id.0]
    }
}

fn load_mesh(device: &Device, mesh: Mesh) -> MeshBuffers {
    println!("loading mesh");
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
    MeshBuffers {
        vertex,
        index,
        index_range,
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub const ATTRIB: [VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub const BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &Self::ATTRIB,
    };
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MeshId(usize);

pub trait MeshProvider: 'static + Copy {
    fn create_mesh() -> Mesh;
}

pub struct MeshBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
    pub index_range: Range<u32>,
}

impl PartialEq for MeshBuffers {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for MeshBuffers {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for MeshBuffers {}

impl Ord for MeshBuffers {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self as *const MeshBuffers).cmp(&(other as *const MeshBuffers))
    }
}
