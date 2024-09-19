use std::{any::TypeId, collections::HashMap, ops::Range, sync::Arc};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    Buffer, BufferAddress, BufferSlice, BufferUsages, Device,
    PrimitiveTopology, VertexAttribute, VertexBufferLayout,
};

pub struct MeshManager {
    static_mesh_ids: HashMap<TypeId, MeshId>,
    static_meshes: Vec<MeshBuffers>,
    dynamic_meshes: Vec<MeshBuffers>,
    device: Arc<Device>,
}

impl MeshManager {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            static_mesh_ids: HashMap::new(),
            static_meshes: Vec::new(),
            dynamic_meshes: Vec::new(),
            device,
        }
    }

    pub(crate) fn get_by_id(&self, id: MeshId) -> &MeshBuffers {
        if id.dynamic {
            &self.dynamic_meshes[id.index]
        } else {
            &self.static_meshes[id.index]
        }
    }

    pub fn clear_dynamic(&mut self) {
        self.dynamic_meshes.clear();
    }
}

fn load_mesh(device: &Device, mesh: Mesh<impl Vertex>) -> MeshBuffers {
    let index_range = 0..mesh.indices.len() as u32;
    let vertices: &[u8] = bytemuck::cast_slice(&mesh.vertices);
    let indices: &[u8] = bytemuck::cast_slice(&mesh.indices);
    let vertex_slice_len = vertices.len() as BufferAddress;
    let index_slice_len = indices.len() as BufferAddress;
    let mut size = vertex_slice_len + index_slice_len;
    // align size to COPY_BUFFER_ALIGNMENT
    size += wgpu::COPY_BUFFER_ALIGNMENT - size % wgpu::COPY_BUFFER_ALIGNMENT;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: BufferUsages::VERTEX | BufferUsages::INDEX,
        mapped_at_creation: true,
    });
    let vertex_slice_range = 0..vertex_slice_len;
    let index_slice_range =
        vertex_slice_len..vertex_slice_len + index_slice_len;
    buffer
        .slice(vertex_slice_range.clone())
        .get_mapped_range_mut()
        .clone_from_slice(vertices);
    buffer
        .slice(index_slice_range.clone())
        .get_mapped_range_mut()
        .clone_from_slice(indices);
    buffer.unmap();
    MeshBuffers {
        buffer,
        vertex_slice_range,
        index_slice_range,
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
    pub indices: Vec<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshId {
    dynamic: bool,
    index: usize,
    pub vtx_type_id: TypeId,
}

pub trait MeshKind<Provider: MeshProvider<Kind = Self> + ?Sized> {
    fn get_or_insert(manager: &mut MeshManager, provider: Provider) -> MeshId;
}

pub struct Static;

impl<Provider: MeshProvider<Kind = Static> + 'static + Sized> MeshKind<Provider>
    for Static
{
    fn get_or_insert(manager: &mut MeshManager, provider: Provider) -> MeshId {
        *manager
            .static_mesh_ids
            .entry(TypeId::of::<Provider>())
            .or_insert_with(|| {
                let index = manager.static_meshes.len();
                manager
                    .static_meshes
                    .push(load_mesh(&manager.device, provider.create_mesh()));
                MeshId {
                    dynamic: false,
                    index,
                    vtx_type_id: TypeId::of::<Provider::Vertex>(),
                }
            })
    }
}
pub struct Dynamic;

impl<Provider: MeshProvider<Kind = Dynamic>> MeshKind<Provider> for Dynamic {
    fn get_or_insert(manager: &mut MeshManager, provider: Provider) -> MeshId {
        let index = manager.dynamic_meshes.len();
        manager
            .dynamic_meshes
            .push(load_mesh(&manager.device, provider.create_mesh()));
        MeshId {
            dynamic: true,
            index,
            vtx_type_id: TypeId::of::<Provider::Vertex>(),
        }
    }
}

pub trait MeshProvider {
    type Vertex: Vertex;
    type Kind: MeshKind<Self>;
    fn create_mesh(self) -> Mesh<Self::Vertex>;
}

pub struct MeshBuffers {
    pub buffer: Buffer,
    pub vertex_slice_range: Range<u64>,
    pub index_slice_range: Range<u64>,
    pub index_range: Range<u32>,
}

impl MeshBuffers {
    pub fn vertex_slice(&self) -> BufferSlice {
        self.buffer.slice(self.vertex_slice_range.clone())
    }

    pub fn index_slice(&self) -> BufferSlice {
        self.buffer.slice(self.index_slice_range.clone())
    }
}
