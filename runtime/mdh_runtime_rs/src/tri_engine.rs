use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;
use wgpu::SurfaceError;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::platform::pump_events::EventLoopExtPumpEvents;
#[cfg(all(
    unix,
    not(target_os = "android"),
    not(target_os = "ios"),
    not(target_os = "macos"),
    not(target_family = "wasm"),
    not(target_os = "redox")
))]
use winit::platform::wayland::EventLoopBuilderExtWayland;
#[cfg(all(
    unix,
    not(target_os = "android"),
    not(target_os = "ios"),
    not(target_os = "macos"),
    not(target_family = "wasm"),
    not(target_os = "redox")
))]
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::{Window, WindowBuilder};

thread_local! {
    static ENGINE: RefCell<TriEngine> = RefCell::new(TriEngine::new());
}

pub fn with_engine<F, R>(f: F) -> R
where
    F: FnOnce(&mut TriEngine) -> R,
{
    ENGINE.with(|engine| f(&mut *engine.borrow_mut()))
}

pub type LoopCallback = Box<dyn FnMut(f64)>;

static TRI_DEBUG_EMPTY_SCENE_LOGGED: AtomicBool = AtomicBool::new(false);
static TRI_DEBUG_EMPTY_BATCH_LOGGED: AtomicBool = AtomicBool::new(false);
static TRI_DEBUG_DRAW_LOGGED: AtomicBool = AtomicBool::new(false);

fn tri_debug_enabled() -> bool {
    env::var_os("MDH_TRI_DEBUG").is_some()
}

fn build_event_loop(backend: Option<&str>) -> Result<EventLoop<()>, String> {
    let mut builder = EventLoopBuilder::new();
    #[cfg(all(
        unix,
        not(target_os = "android"),
        not(target_os = "ios"),
        not(target_os = "macos"),
        not(target_family = "wasm"),
        not(target_os = "redox")
    ))]
    {
        match backend {
            Some("x11") => builder.with_x11(),
            Some("wayland") => builder.with_wayland(),
            _ => &mut builder,
        };
    }
    builder
        .build()
        .map_err(|e| format!("event loop: {e:?}"))
}

fn create_event_loop() -> Result<EventLoop<()>, String> {
    let backend = env::var("MDH_TRI_BACKEND")
        .or_else(|_| env::var("WINIT_UNIX_BACKEND"))
        .ok()
        .map(|val| val.to_ascii_lowercase());
    let backend_hint = backend.as_deref();
    match build_event_loop(backend_hint) {
        Ok(loop_handle) => Ok(loop_handle),
        Err(err) => {
            #[cfg(all(
                unix,
                not(target_os = "android"),
                not(target_os = "ios"),
                not(target_os = "macos"),
                not(target_family = "wasm"),
                not(target_os = "redox")
            ))]
            {
                let err_lower = err.to_ascii_lowercase();
                if backend_hint.is_none()
                    && err_lower.contains("waylanderror")
                    && err_lower.contains("nocompositor")
                {
                    if tri_debug_enabled() {
                        eprintln!("tri: wayland init failed, falling back to x11");
                    }
                    if let Ok(loop_handle) = build_event_loop(Some("x11")) {
                        return Ok(loop_handle);
                    }
                }
            }
            Err(err)
        }
    }
}

pub struct MeshData {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

pub struct RenderItem {
    pub mesh: MeshData,
    pub mesh_key: Option<usize>,
    pub object_key: Option<usize>,
    pub model: Mat4,
    pub color: [f32; 4],
    pub ambient: [f32; 4],
    pub light_dir: [f32; 4],
    pub light_color: [f32; 4],
    pub point_pos: [f32; 4],
    pub point_color: [f32; 4],
    pub point_params: [f32; 4],
    pub mat_params: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    color: [f32; 4],
    ambient: [f32; 4],
    light_dir: [f32; 4],
    light_color: [f32; 4],
    point_pos: [f32; 4],
    point_color: [f32; 4],
    point_params: [f32; 4],
    mat_params: [f32; 4],
}

const TRI_SHADER: &str = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    color: vec4<f32>,
    ambient: vec4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
    point_pos: vec4<f32>,
    point_color: vec4<f32>,
    point_params: vec4<f32>,
    mat_params: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world = uniforms.model * vec4<f32>(in.position, 1.0);
    out.position = uniforms.view_proj * world;
    out.normal = normalize((uniforms.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_pos = world.xyz;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let ld = uniforms.light_dir.xyz;
    let has_light = length(ld) > 0.0001;
    let l = normalize(-ld);
    let diff = max(dot(n, l), 0.0);
    let diffuse = select(0.0, diff, has_light);
    var lighting = uniforms.ambient.rgb + diffuse * uniforms.light_color.rgb;

    let point_color = uniforms.point_color.rgb;
    if length(point_color) > 0.0001 {
        let point_vec = uniforms.point_pos.xyz - in.world_pos;
        let dist = length(point_vec);
        let pd = normalize(point_vec);
        let point_diff = max(dot(n, pd), 0.0);
        var att = 1.0;
        if uniforms.point_params.y > 0.0001 {
            att = 1.0 / (1.0 + uniforms.point_params.y * dist * dist);
        }
        if uniforms.point_params.x > 0.0001 && dist > uniforms.point_params.x {
            att = 0.0;
        }
        lighting = lighting + point_diff * point_color * att;
    }
    let metal = clamp(uniforms.mat_params.x, 0.0, 1.0);
    let rough = clamp(uniforms.mat_params.y, 0.0, 1.0);
    let diffuse_scale = (1.0 - metal * 0.6) * (1.0 - rough * 0.4);
    lighting = lighting * diffuse_scale;
    let color = uniforms.color.rgb * lighting;
    return vec4<f32>(color, uniforms.color.a);
}
"#;

#[derive(Debug)]
pub struct RendererState {
    window: &'static Window,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline_fill: wgpu::RenderPipeline,
    pipeline_wireframe: Option<wgpu::RenderPipeline>,
    uniform_layout: wgpu::BindGroupLayout,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    pixel_ratio: f32,
}

#[derive(Debug)]
struct GpuMesh {
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    index_count: u32,
}

#[derive(Debug)]
struct UniformEntry {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl RendererState {
    fn new(window: &'static Window) -> Result<Self, String> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("create_surface failed: {e:?}"))?;
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ))
        .ok_or_else(|| "No suitable GPU adapter found".to_string())?;
        let mut required_features = wgpu::Features::empty();
        if adapter
            .features()
            .contains(wgpu::Features::POLYGON_MODE_LINE)
        {
            required_features |= wgpu::Features::POLYGON_MODE_LINE;
        }
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("tri_device"),
                required_features,
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .map_err(|e| format!("request_device failed: {e:?}"))?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("tri_shader"),
            source: wgpu::ShaderSource::Wgsl(TRI_SHADER.into()),
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tri_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<Uniforms>() as u64,
                    ),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("tri_pipeline_layout"),
            bind_group_layouts: &[&uniform_layout],
            push_constant_ranges: &[],
        });

        let pipeline_fill = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("tri_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let pipeline_wireframe = if required_features.contains(wgpu::Features::POLYGON_MODE_LINE) {
            Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("tri_pipeline_wire"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Line,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            }))
        } else {
            None
        };

        let (depth_texture, depth_view) = create_depth_texture(&device, &config);

        Ok(RendererState {
            window,
            surface,
            device,
            queue,
            config,
            pipeline_fill,
            pipeline_wireframe,
            uniform_layout,
            depth_texture,
            depth_view,
            pixel_ratio: 1.0,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        let (depth_texture, depth_view) = create_depth_texture(&self.device, &self.config);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;
    }

    fn set_pixel_ratio(&mut self, ratio: f32) {
        self.pixel_ratio = ratio.max(0.1);
    }

    fn render(&mut self) -> Result<(), String> {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
                return Ok(());
            }
            Err(SurfaceError::OutOfMemory) => {
                return Err("Surface out of memory".to_string());
            }
            Err(SurfaceError::Timeout) => return Ok(()),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("tri_render"),
            },
        );
        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tri_clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.1,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    fn render_scene(
        &mut self,
        view_proj: Mat4,
        items: &[RenderItem],
        mesh_cache: &mut HashMap<usize, GpuMesh>,
        uniform_cache: &mut HashMap<usize, UniformEntry>,
        wireframe: bool,
    ) -> Result<(), String> {
        if items.is_empty() {
            if tri_debug_enabled() && !TRI_DEBUG_EMPTY_SCENE_LOGGED.swap(true, Ordering::Relaxed) {
                eprintln!("tri: render_scene with 0 items");
            }
            return self.render();
        }

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
                return Ok(());
            }
            Err(SurfaceError::OutOfMemory) => {
                return Err("Surface out of memory".to_string());
            }
            Err(SurfaceError::Timeout) => return Ok(()),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        enum DrawSource {
            Cached(usize),
            Owned {
                vertex: wgpu::Buffer,
                index: wgpu::Buffer,
                index_count: u32,
            },
        }

        enum UniformSource {
            Cached(usize),
            Owned(wgpu::BindGroup),
        }

        struct DrawBatch {
            source: DrawSource,
            uniform: UniformSource,
        }

        let mut batches = Vec::with_capacity(items.len());
        for item in items {
            let source = if let Some(key) = item.mesh_key {
                if !mesh_cache.contains_key(&key) {
            let vertices: Vec<Vertex> = item
                .mesh
                .vertices
                .iter()
                .enumerate()
                .map(|(idx, position)| Vertex {
                    position: *position,
                    normal: item
                        .mesh
                        .normals
                        .get(idx)
                        .copied()
                        .unwrap_or([0.0, 1.0, 0.0]),
                })
                .collect();
                    if vertices.is_empty() || item.mesh.indices.is_empty() {
                        continue;
                    }

                    let vertex = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("tri_vertices"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    let index = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("tri_indices"),
                        contents: bytemuck::cast_slice(&item.mesh.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });
                    let index_count = item.mesh.indices.len() as u32;
                    mesh_cache.insert(
                        key,
                        GpuMesh {
                            vertex,
                            index,
                            index_count,
                        },
                    );
                }
                DrawSource::Cached(key)
            } else {
            let vertices: Vec<Vertex> = item
                .mesh
                .vertices
                .iter()
                .enumerate()
                .map(|(idx, position)| Vertex {
                    position: *position,
                    normal: item
                        .mesh
                        .normals
                        .get(idx)
                        .copied()
                        .unwrap_or([0.0, 1.0, 0.0]),
                })
                .collect();
                if vertices.is_empty() || item.mesh.indices.is_empty() {
                    continue;
                }

                let vertex = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tri_vertices"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tri_indices"),
                    contents: bytemuck::cast_slice(&item.mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                let index_count = item.mesh.indices.len() as u32;
                DrawSource::Owned {
                    vertex,
                    index,
                    index_count,
                }
            };

            let uniforms = Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: item.model.to_cols_array_2d(),
                color: item.color,
                ambient: item.ambient,
                light_dir: item.light_dir,
                light_color: item.light_color,
                point_pos: item.point_pos,
                point_color: item.point_color,
                point_params: item.point_params,
                mat_params: item.mat_params,
            };

            let uniform = if let Some(key) = item.object_key {
                if !uniform_cache.contains_key(&key) {
                    let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("tri_uniforms"),
                        size: std::mem::size_of::<Uniforms>() as u64,
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("tri_bind_group"),
                        layout: &self.uniform_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniform_buffer.as_entire_binding(),
                        }],
                    });
                    uniform_cache.insert(
                        key,
                        UniformEntry {
                            buffer: uniform_buffer,
                            bind_group,
                        },
                    );
                }
                if let Some(entry) = uniform_cache.get(&key) {
                    self.queue
                        .write_buffer(&entry.buffer, 0, bytemuck::bytes_of(&uniforms));
                }
                UniformSource::Cached(key)
            } else {
                let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tri_uniforms"),
                    contents: bytemuck::bytes_of(&uniforms),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("tri_bind_group"),
                    layout: &self.uniform_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    }],
                });
                UniformSource::Owned(bind_group)
            };

            batches.push(DrawBatch { source, uniform });
        }

        if tri_debug_enabled() && batches.is_empty() {
            if !TRI_DEBUG_EMPTY_BATCH_LOGGED.swap(true, Ordering::Relaxed) {
                eprintln!(
                    "tri: no drawable batches (items={}, wireframe={})",
                    items.len(),
                    wireframe
                );
            }
        }
        if tri_debug_enabled() && !batches.is_empty() {
            if !TRI_DEBUG_DRAW_LOGGED.swap(true, Ordering::Relaxed) {
                eprintln!(
                    "tri: draw batches ready (items={}, batches={}, wireframe={})",
                    items.len(),
                    batches.len(),
                    wireframe
                );
            }
        }

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("tri_render_scene"),
            },
        );
        let pipeline = if wireframe {
            self.pipeline_wireframe.as_ref().unwrap_or(&self.pipeline_fill)
        } else {
            &self.pipeline_fill
        };

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tri_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.1,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            rpass.set_pipeline(pipeline);
            for batch in &batches {
                match &batch.uniform {
                    UniformSource::Cached(key) => {
                        if let Some(entry) = uniform_cache.get(key) {
                            rpass.set_bind_group(0, &entry.bind_group, &[]);
                        } else {
                            continue;
                        }
                    }
                    UniformSource::Owned(bind_group) => {
                        rpass.set_bind_group(0, bind_group, &[]);
                    }
                }
                match &batch.source {
                    DrawSource::Cached(key) => {
                        if let Some(mesh) = mesh_cache.get(key) {
                            rpass.set_vertex_buffer(0, mesh.vertex.slice(..));
                            rpass.set_index_buffer(
                                mesh.index.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                        }
                    }
                    DrawSource::Owned {
                        vertex,
                        index,
                        index_count,
                    } => {
                        rpass.set_vertex_buffer(0, vertex.slice(..));
                        rpass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint32);
                        rpass.draw_indexed(0..*index_count, 0, 0..1);
                    }
                }
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> (wgpu::Texture, wgpu::TextureView) {
    let size = wgpu::Extent3d {
        width: config.width.max(1),
        height: config.height.max(1),
        depth_or_array_layers: 1,
    };
    let desc = wgpu::TextureDescriptor {
        label: Some("tri_depth"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let texture = device.create_texture(&desc);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[derive(Debug, Default)]
pub struct TriEngine {
    next_renderer: usize,
    renderers: HashMap<usize, RendererState>,
    mesh_cache: HashMap<usize, GpuMesh>,
    uniform_cache: HashMap<usize, UniformEntry>,
    event_loop: Option<EventLoop<()>>,
}

impl TriEngine {
    pub fn new() -> Self {
        TriEngine {
            next_renderer: 1,
            renderers: HashMap::new(),
            mesh_cache: HashMap::new(),
            uniform_cache: HashMap::new(),
            event_loop: None,
        }
    }

    pub fn create_renderer(&mut self) -> Result<usize, String> {
        if self.event_loop.is_none() {
            self.event_loop = Some(create_event_loop()?);
        }
        let event_loop = self
            .event_loop
            .as_ref()
            .ok_or_else(|| "Event loop unavailable".to_string())?;
        let window = WindowBuilder::new()
            .with_title("mdhavers tri")
            .build(event_loop)
            .map_err(|e| format!("window build failed: {e:?}"))?;
        let window = Box::leak(Box::new(window));
        let renderer = RendererState::new(window)?;

        let handle = self.next_renderer;
        self.next_renderer = self.next_renderer.saturating_add(1);
        self.renderers.insert(handle, renderer);
        Ok(handle)
    }

    pub fn set_size(&mut self, handle: usize, width: u32, height: u32) {
        if let Some(renderer) = self.renderers.get_mut(&handle) {
            renderer.resize(width, height);
        }
    }

    pub fn set_pixel_ratio(&mut self, handle: usize, ratio: f32) {
        if let Some(renderer) = self.renderers.get_mut(&handle) {
            renderer.set_pixel_ratio(ratio);
        }
    }

    pub fn render_scene(
        &mut self,
        handle: usize,
        view_proj: Mat4,
        items: Vec<RenderItem>,
        wireframe: bool,
    ) -> Result<(), String> {
        self.pump_events(handle);
        if let Some(renderer) = self.renderers.get_mut(&handle) {
            renderer.render_scene(
                view_proj,
                &items,
                &mut self.mesh_cache,
                &mut self.uniform_cache,
                wireframe,
            )?;
        }
        Ok(())
    }

    fn pump_events(&mut self, handle: usize) {
        let event_loop = match self.event_loop.as_mut() {
            Some(loop_ref) => loop_ref,
            None => return,
        };
        let renderer = match self.renderers.get_mut(&handle) {
            Some(renderer) => renderer,
            None => return,
        };
        let window_id = renderer.window.id();
        let _ = event_loop.pump_events(Some(Duration::from_millis(0)), |event, _target| {
            match event {
                Event::WindowEvent { window_id: id, event } if id == window_id => match event {
                    WindowEvent::Resized(size) => {
                        renderer.resize(size.width, size.height);
                    }
                    WindowEvent::ScaleFactorChanged {
                        mut inner_size_writer,
                        ..
                    } => {
                        let size = renderer.window.inner_size();
                        let _ = inner_size_writer.request_inner_size(size);
                        renderer.resize(size.width, size.height);
                    }
                    WindowEvent::CloseRequested => {}
                    _ => {}
                },
                Event::AboutToWait => {
                    renderer.window.request_redraw();
                }
                _ => {}
            }
        });
    }

    pub fn remove_mesh(&mut self, key: usize) {
        self.mesh_cache.remove(&key);
    }

    pub fn remove_uniform(&mut self, key: usize) {
        self.uniform_cache.remove(&key);
    }

    pub fn remove_renderer(&mut self, handle: usize) {
        self.renderers.remove(&handle);
    }

    pub fn run_loop(
        &mut self,
        handle: usize,
        mut callback: Option<LoopCallback>,
    ) -> Result<(), String> {
        if self.event_loop.is_none() {
            self.event_loop = Some(create_event_loop()?);
        }
        let event_loop = self
            .event_loop
            .take()
            .ok_or_else(|| "Event loop unavailable".to_string())?;
        let renderer_ptr = self
            .renderers
            .get_mut(&handle)
            .map(|renderer| renderer as *mut RendererState)
            .ok_or_else(|| "Unknown renderer handle".to_string())?;
        let window_id = unsafe { (*renderer_ptr).window.id() };
        let mut last_frame = Instant::now();

        event_loop
            .run(move |event, target| {
                target.set_control_flow(ControlFlow::Poll);
                match event {
                    Event::WindowEvent { window_id: id, event } if id == window_id => match event {
                        WindowEvent::CloseRequested => target.exit(),
                        WindowEvent::Resized(size) => unsafe {
                            (*renderer_ptr).resize(size.width, size.height);
                        },
                        WindowEvent::ScaleFactorChanged {
                            mut inner_size_writer,
                            ..
                        } => unsafe {
                            let size = (*renderer_ptr).window.inner_size();
                            let _ = inner_size_writer.request_inner_size(size);
                            (*renderer_ptr).resize(size.width, size.height);
                        },
                        WindowEvent::RedrawRequested => {
                            let now = Instant::now();
                            let dt = now.duration_since(last_frame).as_secs_f64();
                            last_frame = now;
                            if let Some(cb) = callback.as_mut() {
                                cb(dt);
                            } else if unsafe { (*renderer_ptr).render() }.is_err() {
                                target.exit();
                            }
                        }
                        _ => {}
                    },
                    Event::AboutToWait => {
                        unsafe { (*renderer_ptr).window.request_redraw() };
                    }
                    _ => {}
                }
            })
            .map_err(|e| format!("event loop error: {e:?}"))
    }
}
