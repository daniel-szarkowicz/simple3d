use std::{borrow::Cow, num::NonZero, sync::Arc};

use bytemuck::Zeroable;
use pollster::FutureExt;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, BufferDescriptor, BufferUsages, Color,
    CommandEncoderDescriptor, DepthBiasState, DepthStencilState, Device,
    Extent3d, IndexFormat, PresentMode, Queue,
    RenderPassDepthStencilAttachment, RenderPipeline, StencilState, Surface,
    SurfaceConfiguration, Texture, TextureDescriptor, TextureView,
    TextureViewDescriptor, VertexAttribute, VertexBufferLayout,
};
use winit::{
    dpi::PhysicalSize, event::Event, event_loop::ActiveEventLoop,
    window::Window,
};

use crate::{
    camera::FirstPersonCamera,
    canvas::{Canvas, DrawCommand},
    mesh::{MeshManager, Vertex},
};

pub struct Context {
    window: Arc<Window>,
    config: SurfaceConfiguration,
    surface: Surface<'static>,
    device: Arc<Device>,
    render_pipeline: RenderPipeline,
    queue: Queue,
    camera_uniform_buffer: Buffer,
    camera_bind_group: BindGroup,
    camera: FirstPersonCamera,
    meshes: MeshManager,
    depth_texture: Texture,
    depth_texture_view: TextureView,
    instance_buffer: Buffer,
    instances: Instances,
}

impl Context {
    pub(crate) fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window.clone()).unwrap();

        let (adapter, device, queue) = async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                })
                .await
                .unwrap();
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::empty(),
                        required_limits:
                            wgpu::Limits::downlevel_webgl2_defaults()
                                .using_resolution(adapter.limits()),
                        memory_hints: wgpu::MemoryHints::MemoryUsage,
                    },
                    None,
                )
                .await
                .unwrap();
            (adapter, Arc::new(device), queue)
        }
        .block_on();

        let shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "shader.wgsl"
                ))),
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let camera = FirstPersonCamera::default();

        let camera_uniform_data = CameraUniform::from(&camera);

        let camera_uniform_buffer =
            device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&camera_uniform_data),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: size_of::<Instances>() as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::BUFFER_LAYOUT,
                        InstanceData::BUFFER_LAYOUT,
                    ],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..wgpu::PrimitiveState::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                multiview: None,
                cache: None,
            });

        let camera_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                }],
            });

        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        config.present_mode = PresentMode::Fifo;
        let (depth_texture, depth_texture_view) =
            create_depth_texture(&device, &config);

        surface.configure(&device, &config);
        let meshes = MeshManager::new(device.clone());
        Self {
            window: window.clone(),
            config,
            surface,
            device,
            render_pipeline,
            queue,
            camera_uniform_buffer,
            camera_bind_group,
            camera,
            meshes,
            depth_texture,
            depth_texture_view,
            instance_buffer,
            instances: Zeroable::zeroed(),
        }
    }

    pub fn create_canvas(&mut self) -> Canvas {
        Canvas::new(&mut self.meshes)
    }

    pub(crate) fn render(&mut self, mut commands: Vec<DrawCommand>) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let camera_uniform_data = CameraUniform::from(&self.camera);
        self.queue
            .write_buffer_with(
                &self.camera_uniform_buffer,
                0,
                NonZero::new(size_of::<CameraUniform>() as u64).unwrap(),
            )
            .unwrap()
            .copy_from_slice(bytemuck::bytes_of(&camera_uniform_data));
        let mut color_load_op = wgpu::LoadOp::Clear(Color {
            r: 100.0 / 255.0,
            g: 149.0 / 255.0,
            b: 237.0 / 255.0,
            a: 1.0,
        });
        let mut depth_load_op = wgpu::LoadOp::Clear(1.0);
        commands.sort_unstable_by(|c1, c2| c1.mesh.cmp(&c2.mesh));
        // TODO: store meshids in DrawCommand to fix this
        let mut curr = commands[0].mesh.clone();
        for batch in commands.split_inclusive(|cmd| {
            if cmd.mesh != curr {
                curr = cmd.mesh.clone();
                true
            } else {
                false
            }
        }) {
            for (i, cmd) in batch.iter().enumerate() {
                self.instances[i] = cmd.into();
            }
            let instances: &[u8] =
                bytemuck::cast_slice(&self.instances[0..batch.len()]);
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor::default());
            self.queue
                .write_buffer_with(
                    &self.instance_buffer,
                    0,
                    NonZero::new(instances.len() as u64).unwrap(),
                )
                .unwrap()
                .copy_from_slice(instances);
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: color_load_op,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: Some(
                        RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view,
                            depth_ops: Some(wgpu::Operations {
                                load: depth_load_op,
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        },
                    ),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            rpass.set_vertex_buffer(0, batch[0].mesh.vertex.slice(..));
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            rpass.set_index_buffer(
                batch[0].mesh.index.slice(..),
                IndexFormat::Uint16,
            );
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.draw_indexed(
                batch[0].mesh.index_range.clone(),
                0,
                0..batch.len() as u32,
            );
            drop(rpass);
            self.queue.submit(Some(encoder.finish()));
            color_load_op = wgpu::LoadOp::Load;
            depth_load_op = wgpu::LoadOp::Load;
        }
        frame.present();
        self.window.request_redraw();
    }

    pub fn event(&mut self, event: &Event<()>) {
        self.camera.event(event);
    }

    pub fn update_camera(&mut self, dt: f32) {
        self.camera.update(dt);
    }

    pub(crate) fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        (self.depth_texture, self.depth_texture_view) =
            create_depth_texture(&self.device, &self.config);
        self.surface.configure(&self.device, &self.config);
    }
}

fn create_depth_texture(
    device: &Device,
    config: &SurfaceConfiguration,
) -> (Texture, TextureView) {
    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    position: [f32; 3],
    _padding: [u8; 4],
}

impl From<&FirstPersonCamera> for CameraUniform {
    fn from(camera: &FirstPersonCamera) -> Self {
        Self {
            view_proj: camera.view_proj().into(),
            position: camera.position().into(),
            _padding: [0; 4],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    model: [[f32; 4]; 4],
    model_inv: [[f32; 4]; 4],
    color: [f32; 3],
}

type Instances = [InstanceData; 256];

impl InstanceData {
    pub const ATTRIB: [VertexAttribute; 9] = wgpu::vertex_attr_array![
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
        8 => Float32x4,
        9 => Float32x4,
        10 => Float32x4,
        11 => Float32x4,
        12 => Float32x3,
    ];

    pub const BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<InstanceData>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &Self::ATTRIB,
    };
}

impl From<&DrawCommand> for InstanceData {
    fn from(command: &DrawCommand) -> Self {
        Self {
            model: command.transform.into(),
            model_inv: command.transform.try_inverse().unwrap().into(),
            color: command.color,
        }
    }
}
