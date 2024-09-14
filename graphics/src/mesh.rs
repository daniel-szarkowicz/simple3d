use std::{any::TypeId, collections::HashMap, ops::Range, sync::Arc};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, PrimitiveTopology, VertexAttribute,
    VertexBufferLayout,
};

// VertexType -> index
//

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

    pub fn get_or_insert<T: MeshProvider>(&mut self) -> MeshId {
        *self.mesh_ids.entry(TypeId::of::<T>()).or_insert_with(|| {
            let id = self.meshes.len();
            self.meshes.push(load_mesh(&self.device, T::create_mesh()));
            MeshId(id, TypeId::of::<T::Vertex>())
        })
    }

    pub(crate) fn get_by_id(&self, id: MeshId) -> &MeshBuffers {
        &self.meshes[id.0]
    }
}

fn load_mesh(device: &Device, mesh: Mesh<impl Vertex>) -> MeshBuffers {
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

pub trait Vertex: Pod {
    const ATTRIBUTES: &'static [VertexAttribute];
    const BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: Self::ATTRIBUTES,
    };
    const PRIMITIVE_TOPOLOGY: PrimitiveTopology;
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct PNVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex for PNVertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    const PRIMITIVE_TOPOLOGY: PrimitiveTopology =
        PrimitiveTopology::TriangleList;
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct PDVertex {
    pub position: [f32; 3],
    pub direction: [f32; 3],
}

impl Vertex for PDVertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    const PRIMITIVE_TOPOLOGY: PrimitiveTopology = PrimitiveTopology::LineList;
}

pub struct Mesh<V: Vertex> {
    pub vertices: Vec<V>,
    pub indices: Vec<u16>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshId(usize, pub TypeId);

pub trait MeshProvider: 'static + Copy {
    type Vertex: Vertex;
    fn create_mesh() -> Mesh<Self::Vertex>;
}

pub struct MeshBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
    pub index_range: Range<u32>,
}
