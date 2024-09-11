use std::{borrow::Cow, cell::RefCell, num::NonZero, sync::Arc, time::Instant};

use nalgebra::Matrix4;
use pollster::FutureExt;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt, RenderEncoder},
    BindGroup, Buffer, BufferUsages, CommandEncoderDescriptor, Device,
    PresentMode, Queue, RenderPipeline, Surface, SurfaceConfiguration,
    TextureDescriptor, TextureViewDescriptor, VertexAttribute,
    VertexBufferLayout,
};
use winit::{
    dpi::PhysicalSize, event::Event, event_loop::ActiveEventLoop,
    window::Window,
};

use crate::{
    camera::FirstPersonCamera,
    canvas::{Canvas, DrawCommand},
    mesh::Mesh,
    shader::Shader,
};

pub struct Context {
    window: Arc<Window>,
    config: SurfaceConfiguration,
    surface: Surface<'static>,
    device: Arc<Device>,
    render_pipeline: RenderPipeline,
    queue: Queue,
    vertex_buffer: Buffer,
    camera_uniform_buffer: Buffer,
    model_uniform_buffer: Buffer,
    camera_bind_group: BindGroup,
    model_bind_group: BindGroup,
    start: Instant,
    camera: FirstPersonCamera,
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

        let model_bind_group_layout =
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
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &model_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        #[rustfmt::skip]
        let triangle = [
            0.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            1.0, 0.0, 0.0f32
        ];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&triangle),
            usage: BufferUsages::VERTEX,
        });

        let camera = FirstPersonCamera::default();

        let camera_uniform_buffer =
            device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(camera.view_proj().as_slice()),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let model_uniform_buffer =
            device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(
                    Matrix4::<f32>::identity().as_slice(),
                ),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    // buffers: &[],
                    buffers: &[VertexBufferLayout {
                        array_stride: size_of::<[f32; 3]>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
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

        let model_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &model_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_uniform_buffer.as_entire_binding(),
                }],
            });

        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        config.present_mode = PresentMode::Fifo;

        surface.configure(&device, &config);
        Self {
            window: window.clone(),
            config,
            surface,
            device,
            render_pipeline,
            queue,
            vertex_buffer,
            camera_uniform_buffer,
            model_uniform_buffer,
            camera_bind_group,
            model_bind_group,
            start: Instant::now(),
            camera,
        }
    }

    // pub fn load_mesh(&self, mesh: Mesh) -> MeshId {
    //     let mut meshes = self.meshes.borrow_mut();
    //     let id = MeshId(meshes.len() as u32);
    //     meshes.push(mesh);
    //     id
    // }

    // pub fn load_shader(&self, shader: Shader) -> ShaderId {
    //     let mut shaders = self.shaders.borrow_mut();
    //     let id = ShaderId(shaders.len() as u32);
    //     shaders.push(shader);
    //     id
    // }

    // pub fn render(&self, commands: Vec<DrawCommand>) {
    //     let meshes = self.meshes.borrow();
    //     let shaders = self.shaders.borrow();
    //     for command in commands {
    //         println!(
    //             "{} {}",
    //             meshes[command.mesh.0 as usize].name,
    //             shaders[command.shader.0 as usize].name
    //         );
    //     }
    // }

    pub fn create_canvas(&self) -> Canvas {
        Canvas::new()
    }

    pub(crate) fn render(&self, commands: Vec<DrawCommand>) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let view_proj = self.camera.view_proj();
        let view_proj = bytemuck::cast_slice(view_proj.as_slice());
        for command in commands {
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor::default());
            self.queue
                .write_buffer_with(
                    &self.camera_uniform_buffer,
                    0,
                    NonZero::new(view_proj.len() as u64).unwrap(),
                )
                .unwrap()
                .copy_from_slice(view_proj);
            let transform = bytemuck::cast_slice(command.transform.as_slice());
            self.queue
                .write_buffer_with(
                    &self.model_uniform_buffer,
                    0,
                    NonZero::new(transform.len() as u64).unwrap(),
                )
                .unwrap()
                .copy_from_slice(transform);
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_bind_group(1, &self.model_bind_group, &[]);
            // for command in commands {
            //     println!("{:?}", command.transform);
            //     let transform =
            //         bytemuck::cast_slice(command.transform.as_slice());
            //     // self.queue.write_buffer(
            //     //     &self.model_uniform_buffer,
            //     //     0,
            //     //     transform,
            //     // );
            //     self.queue
            //         .write_buffer_with(
            //             &self.model_uniform_buffer,
            //             0,
            //             NonZero::new(transform.len() as u64).unwrap(),
            //         )
            //         .unwrap()
            //         .copy_from_slice(transform);
            //     rpass.set_bind_group(1, &self.model_bind_group, &[]);
            //     rpass.draw(0..3, 0..1);
            // }
            rpass.draw(0..3, 0..1);
            drop(rpass);
            self.queue.submit(Some(encoder.finish()));
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
        self.surface.configure(&self.device, &self.config);
    }
}

#[derive(Clone, Copy)]
pub struct MeshId(pub u32);

#[derive(Clone, Copy)]
pub struct ShaderId(pub u32);
