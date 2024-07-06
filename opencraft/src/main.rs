#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(duration_millis_float)]

mod camera;
mod core;

use crate::camera::{Camera, Direction};
use crate::core::math::angle::{Angle, FULL_ROTATION};
use crate::core::math::mat4::{self, Mat4x4};
use crate::core::math::vec3::Vec3;
use crate::core::math::{X_AXIS, Z_AXIS};
use anyhow::{anyhow, Result};
use bytemuck::NoUninit;
use image::io::Reader;
use image::GenericImageView;
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::{iter, mem};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
  include_wgsl, vertex_attr_array, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry,
  BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
  Buffer, BufferAddress, BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites,
  CommandEncoderDescriptor, CompareFunction, DepthBiasState, DepthStencilState, Device, Extent3d,
  Face, Features, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout, Instance,
  InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations, Origin3d,
  PipelineLayoutDescriptor, PolygonMode, PowerPreference, PresentMode, PrimitiveState,
  PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
  RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
  SamplerBindingType, SamplerDescriptor, ShaderStages, StencilState, StoreOp, Surface,
  SurfaceConfiguration, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
  TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
  VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowBuilder};

fn main() -> Result<()> {
  pollster::block_on(start())?;
  Ok(())
}

async fn start() -> Result<()> {
  env_logger::init();

  let event_loop = EventLoop::new()?;
  let window = WindowBuilder::new()
    .with_title("Opencraft")
    .build(&event_loop)?;
  let _ = window.set_cursor_grab(CursorGrabMode::Confined);
  window.set_cursor_visible(false);

  let mut app = App::new(&window).await?;

  event_loop.run(|event, target| match event {
    Event::WindowEvent { event, .. } => match event {
      WindowEvent::CloseRequested => {
        target.exit();
      }
      WindowEvent::RedrawRequested => {
        if let Err(err) = app.compose() {
          eprintln!("Error during composition loop: {:?}", err);
          target.exit();
        }
      }
      WindowEvent::Resized(physical_size) => {
        app.resize(physical_size);
      }
      WindowEvent::ScaleFactorChanged { .. } => {
        app.resize(window.inner_size());
      }
      WindowEvent::KeyboardInput {
        event: KeyEvent {
          state,
          physical_key,
          ..
        },
        ..
      } => match state {
        ElementState::Pressed => {
          if let PhysicalKey::Code(code) = physical_key {
            match code {
              KeyCode::Escape => target.exit(),
              code => app.press(code),
            }
          }
        }
        ElementState::Released => {
          if let PhysicalKey::Code(code) = physical_key {
            app.release(code)
          }
        }
      },
      _ => {}
    },
    Event::DeviceEvent {
      event: DeviceEvent::MouseMotion { delta: (x, y) },
      ..
    } => {
      app.motion(x as f32, y as f32);
    }
    Event::AboutToWait => {
      window.request_redraw();
    }
    _ => {}
  })?;

  Ok(())
}

lazy_static! {
  static ref FOV: Angle = Angle::degrees(75.0);
}
const Z_NEAR: f32 = 0.01;
const Z_FAR: f32 = 1000.0;

const CUBE_SIZE: f32 = 1.0;
const CUBE_HALF: f32 = CUBE_SIZE / 2.0;
const CUBE_TRANSLATE: Vec3 = Vec3::new(0.0, 0.0, 3.0);

const BACK: f32 = CUBE_HALF;
const FRONT: f32 = -CUBE_HALF;
const BOTTOM: f32 = -CUBE_HALF;
const TOP: f32 = CUBE_HALF;
const LEFT: f32 = -CUBE_HALF;
const RIGHT: f32 = CUBE_HALF;

const TEX_WIDTH: f32 = 48.0;
const TEX_HEIGHT: f32 = 64.0;

const TEX_FRONT_LEFT: f32 = 16.0 / TEX_WIDTH;
const TEX_FRONT_RIGHT: f32 = 32.0 / TEX_WIDTH;
const TEX_FRONT_TOP: f32 = 32.0 / TEX_HEIGHT;
const TEX_FRONT_BOTTOM: f32 = 48.0 / TEX_HEIGHT;

const TEX_BACK_LEFT: f32 = 16.0 / TEX_WIDTH;
const TEX_BACK_RIGHT: f32 = 32.0 / TEX_WIDTH;
const TEX_BACK_TOP: f32 = 0.0 / TEX_HEIGHT;
const TEX_BACK_BOTTOM: f32 = 16.0 / TEX_HEIGHT;

const TEX_TOP_LEFT: f32 = 16.0 / TEX_WIDTH;
const TEX_TOP_RIGHT: f32 = 32.0 / TEX_WIDTH;
const TEX_TOP_TOP: f32 = 16.0 / TEX_HEIGHT;
const TEX_TOP_BOTTOM: f32 = 32.0 / TEX_HEIGHT;

const TEX_BOTTOM_LEFT: f32 = 16.0 / TEX_WIDTH;
const TEX_BOTTOM_RIGHT: f32 = 32.0 / TEX_WIDTH;
const TEX_BOTTOM_TOP: f32 = 48.0 / TEX_HEIGHT;
const TEX_BOTTOM_BOTTOM: f32 = 64.0 / TEX_HEIGHT;

const TEX_LEFT_LEFT: f32 = 0.0 / TEX_WIDTH;
const TEX_LEFT_RIGHT: f32 = 16.0 / TEX_WIDTH;
const TEX_LEFT_TOP: f32 = 16.0 / TEX_HEIGHT;
const TEX_LEFT_BOTTOM: f32 = 32.0 / TEX_HEIGHT;

const TEX_RIGHT_LEFT: f32 = 32.0 / TEX_WIDTH;
const TEX_RIGHT_RIGHT: f32 = 48.0 / TEX_WIDTH;
const TEX_RIGHT_TOP: f32 = 16.0 / TEX_HEIGHT;
const TEX_RIGHT_BOTTOM: f32 = 32.0 / TEX_HEIGHT;

#[repr(C)]
#[derive(Clone, Copy, NoUninit)]
struct Vertex {
  position: [f32; 3],
  texture_coordinate: [f32; 2],
}

const VERTICES: &[Vertex] = &[
  // Front face
  Vertex {
    position: [LEFT, TOP, FRONT],
    texture_coordinate: [TEX_FRONT_LEFT, TEX_FRONT_TOP],
  },
  Vertex {
    position: [LEFT, BOTTOM, FRONT],
    texture_coordinate: [TEX_FRONT_LEFT, TEX_FRONT_BOTTOM],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    texture_coordinate: [TEX_FRONT_RIGHT, TEX_FRONT_TOP],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    texture_coordinate: [TEX_FRONT_RIGHT, TEX_FRONT_TOP],
  },
  Vertex {
    position: [LEFT, BOTTOM, FRONT],
    texture_coordinate: [TEX_FRONT_LEFT, TEX_FRONT_BOTTOM],
  },
  Vertex {
    position: [RIGHT, BOTTOM, FRONT],
    texture_coordinate: [TEX_FRONT_RIGHT, TEX_FRONT_BOTTOM],
  },
  // Back face
  Vertex {
    position: [LEFT, TOP, BACK],
    texture_coordinate: [TEX_BACK_LEFT, TEX_BACK_BOTTOM],
  },
  Vertex {
    position: [RIGHT, TOP, BACK],
    texture_coordinate: [TEX_BACK_RIGHT, TEX_BACK_BOTTOM],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    texture_coordinate: [TEX_BACK_LEFT, TEX_BACK_TOP],
  },
  Vertex {
    position: [RIGHT, BOTTOM, BACK],
    texture_coordinate: [TEX_BACK_RIGHT, TEX_BACK_TOP],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    texture_coordinate: [TEX_BACK_LEFT, TEX_BACK_TOP],
  },
  Vertex {
    position: [RIGHT, TOP, BACK],
    texture_coordinate: [TEX_BACK_RIGHT, TEX_BACK_BOTTOM],
  },
  // Top face
  Vertex {
    position: [LEFT, TOP, BACK],
    texture_coordinate: [TEX_TOP_LEFT, TEX_TOP_TOP],
  },
  Vertex {
    position: [LEFT, TOP, FRONT],
    texture_coordinate: [TEX_TOP_LEFT, TEX_TOP_BOTTOM],
  },
  Vertex {
    position: [RIGHT, TOP, BACK],
    texture_coordinate: [TEX_TOP_RIGHT, TEX_TOP_TOP],
  },
  Vertex {
    position: [RIGHT, TOP, BACK],
    texture_coordinate: [TEX_TOP_RIGHT, TEX_TOP_TOP],
  },
  Vertex {
    position: [LEFT, TOP, FRONT],
    texture_coordinate: [TEX_TOP_LEFT, TEX_TOP_BOTTOM],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    texture_coordinate: [TEX_TOP_RIGHT, TEX_TOP_BOTTOM],
  },
  // Bottom face
  Vertex {
    position: [RIGHT, BOTTOM, FRONT],
    texture_coordinate: [TEX_BOTTOM_RIGHT, TEX_BOTTOM_TOP],
  },
  Vertex {
    position: [LEFT, BOTTOM, FRONT],
    texture_coordinate: [TEX_BOTTOM_LEFT, TEX_BOTTOM_TOP],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    texture_coordinate: [TEX_BOTTOM_LEFT, TEX_BOTTOM_BOTTOM],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    texture_coordinate: [TEX_BOTTOM_LEFT, TEX_BOTTOM_BOTTOM],
  },
  Vertex {
    position: [RIGHT, BOTTOM, BACK],
    texture_coordinate: [TEX_BOTTOM_RIGHT, TEX_BOTTOM_BOTTOM],
  },
  Vertex {
    position: [RIGHT, BOTTOM, FRONT],
    texture_coordinate: [TEX_BOTTOM_RIGHT, TEX_BOTTOM_TOP],
  },
  // Left face
  Vertex {
    position: [LEFT, TOP, BACK],
    texture_coordinate: [TEX_LEFT_RIGHT, TEX_LEFT_TOP],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    texture_coordinate: [TEX_LEFT_LEFT, TEX_LEFT_TOP],
  },
  Vertex {
    position: [LEFT, TOP, FRONT],
    texture_coordinate: [TEX_LEFT_RIGHT, TEX_LEFT_BOTTOM],
  },
  Vertex {
    position: [LEFT, TOP, FRONT],
    texture_coordinate: [TEX_LEFT_RIGHT, TEX_LEFT_BOTTOM],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    texture_coordinate: [TEX_LEFT_LEFT, TEX_LEFT_TOP],
  },
  Vertex {
    position: [LEFT, BOTTOM, FRONT],
    texture_coordinate: [TEX_LEFT_LEFT, TEX_LEFT_BOTTOM],
  },
  // Right face
  Vertex {
    position: [RIGHT, TOP, BACK],
    texture_coordinate: [TEX_RIGHT_LEFT, TEX_RIGHT_TOP],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    texture_coordinate: [TEX_RIGHT_LEFT, TEX_RIGHT_BOTTOM],
  },
  Vertex {
    position: [RIGHT, BOTTOM, BACK],
    texture_coordinate: [TEX_RIGHT_RIGHT, TEX_RIGHT_TOP],
  },
  Vertex {
    position: [RIGHT, BOTTOM, BACK],
    texture_coordinate: [TEX_RIGHT_RIGHT, TEX_RIGHT_TOP],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    texture_coordinate: [TEX_RIGHT_LEFT, TEX_RIGHT_BOTTOM],
  },
  Vertex {
    position: [RIGHT, BOTTOM, FRONT],
    texture_coordinate: [TEX_RIGHT_RIGHT, TEX_RIGHT_BOTTOM],
  },
];

#[repr(C)]
#[derive(Clone, Copy, NoUninit)]
struct SkyVertex {
  position: [f32; 3],
}

const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

/// Resources that need to be constructed based on the screen's resolution, and
/// therefore reconstructed on resize.
struct ScreenSpaceResources {
  depth_view: TextureView,
}

impl ScreenSpaceResources {
  pub fn construct(device: &Device, config: &SurfaceConfiguration) -> Self {
    let depth_texture = device.create_texture(&TextureDescriptor {
      label: Some("Depth Texture"),
      size: Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: DEPTH_FORMAT,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });

    Self {
      depth_view: depth_texture.create_view(&TextureViewDescriptor::default()),
    }
  }
}

struct App<'a> {
  last: Instant,

  camera: Camera,
  keys_down: HashSet<KeyCode>,
  transform: Mat4x4,
  skybox_transform: Mat4x4,

  surface: Surface<'a>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,

  screen: ScreenSpaceResources,

  transform_buffer: Buffer,
  transform_bind_group: BindGroup,
  pipeline: RenderPipeline,
  vertex_buffer: Buffer,
  grass_bind_group: BindGroup,

  skybox_transform_buffer: Buffer,
  skybox_transform_bind_group: BindGroup,
  skybox_pipeline: RenderPipeline,
  skybox_vertex_buffer: Buffer,
}

impl<'a> App<'a> {
  async fn new(window: &'a Window) -> Result<Self> {
    let instance = Instance::new(InstanceDescriptor {
      backends: Backends::all(),
      ..Default::default()
    });
    let surface = instance.create_surface(window)?;
    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await
      .ok_or_else(|| anyhow!("no compatible adapter available"))?;

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          required_features: Features::empty(),
          required_limits: Limits::default(),
          label: None,
        },
        None,
      )
      .await?;

    let capabilities = surface.get_capabilities(&adapter);
    let surface_format = capabilities
      .formats
      .iter()
      .copied()
      .find(TextureFormat::is_srgb)
      .unwrap_or(capabilities.formats[0]);
    let size = window.inner_size();
    let config = SurfaceConfiguration {
      usage: TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: PresentMode::AutoVsync,
      alpha_mode: capabilities.alpha_modes[0],
      view_formats: Vec::new(),
      desired_maximum_frame_latency: 3,
    };

    surface.configure(&device, &config);

    let grass_image = Reader::open("assets/textures/grass.png")?.decode()?;
    let grass_rgba = grass_image.to_rgba8();
    let (grass_width, grass_height) = grass_image.dimensions();

    let grass_size = Extent3d {
      width: grass_width,
      height: grass_height,
      depth_or_array_layers: 1,
    };
    let grass_texture = device.create_texture(&TextureDescriptor {
      label: Some("Grass Texture"),
      size: grass_size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: TextureFormat::Rgba8UnormSrgb,
      usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      view_formats: &[],
    });

    queue.write_texture(
      ImageCopyTexture {
        texture: &grass_texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
      },
      &grass_rgba,
      ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(4 * grass_size.width),
        rows_per_image: Some(grass_size.height),
      },
      grass_size,
    );

    let grass_texture_view = grass_texture.create_view(&TextureViewDescriptor::default());
    let grass_sampler = device.create_sampler(&SamplerDescriptor::default());
    let grass_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("Grass Bind Group Layout"),
      entries: &[
        BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 1,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    let grass_bind_group = device.create_bind_group(&BindGroupDescriptor {
      label: Some("Grass Bind Group"),
      layout: &grass_bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::TextureView(&grass_texture_view),
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::Sampler(&grass_sampler),
        },
      ],
    });

    let transform = Mat4x4::default();
    let transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Model -> Clip Space Transform Buffer"),
      contents: bytemuck::cast_slice(&[transform]),
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });
    let transform_buffer_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("Transform Buffer Bind Group Layout"),
      entries: &[BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
          ty: BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    });
    let transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
      label: Some("Transform Buffer Bind Group"),
      layout: &transform_buffer_layout,
      entries: &[BindGroupEntry {
        binding: 0,
        resource: transform_buffer.as_entire_binding(),
      }],
    });

    let shader = device.create_shader_module(include_wgsl!("shaders/cube.wgsl"));
    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[&transform_buffer_layout, &grass_bind_group_layout],
      push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&layout),
      vertex: VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[VertexBufferLayout {
          array_stride: mem::size_of::<Vertex>() as BufferAddress,
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x3],
        }],
      },
      fragment: Some(FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(ColorTargetState {
          format: config.format,
          blend: Some(BlendState::REPLACE),
          write_mask: ColorWrites::ALL,
        })],
      }),
      primitive: PrimitiveState {
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: FrontFace::Ccw,
        cull_mode: Some(Face::Back),
        unclipped_depth: false,
        polygon_mode: PolygonMode::Fill,
        conservative: false,
      },
      depth_stencil: Some(DepthStencilState {
        format: DEPTH_FORMAT,
        depth_write_enabled: true,
        depth_compare: CompareFunction::Less,
        stencil: StencilState::default(),
        bias: DepthBiasState::default(),
      }),
      multisample: MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
    });

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(VERTICES),
      usage: BufferUsages::VERTEX,
    });

    let skybox_transform = Mat4x4::default();
    let skybox_transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Skybox Model -> Clip Space Transform Buffer"),
      contents: bytemuck::cast_slice(&[skybox_transform]),
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });
    let skybox_transform_buffer_layout =
      device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Skybox Transform Buffer Bind Group Layout"),
        entries: &[BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::VERTEX,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }],
      });
    let skybox_transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
      label: Some("Skybox Transform Buffer Bind Group"),
      layout: &skybox_transform_buffer_layout,
      entries: &[BindGroupEntry {
        binding: 0,
        resource: skybox_transform_buffer.as_entire_binding(),
      }],
    });

    let skybox_shader = device.create_shader_module(include_wgsl!("shaders/skybox.wgsl"));
    let skybox_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Skybox Render Pipeline Layout"),
      bind_group_layouts: &[&skybox_transform_buffer_layout],
      push_constant_ranges: &[],
    });
    let skybox_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Skybox Render Pipeline"),
      layout: Some(&skybox_layout),
      vertex: VertexState {
        module: &skybox_shader,
        entry_point: "vs_main",
        buffers: &[VertexBufferLayout {
          array_stride: mem::size_of::<SkyVertex>() as BufferAddress,
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x3],
        }],
      },
      fragment: Some(FragmentState {
        module: &skybox_shader,
        entry_point: "fs_main",
        targets: &[Some(ColorTargetState {
          format: config.format,
          blend: Some(BlendState::REPLACE),
          write_mask: ColorWrites::ALL,
        })],
      }),
      primitive: PrimitiveState {
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: FrontFace::Cw,
        cull_mode: Some(Face::Back),
        unclipped_depth: false,
        polygon_mode: PolygonMode::Fill,
        conservative: false,
      },
      depth_stencil: Some(DepthStencilState {
        format: DEPTH_FORMAT,
        depth_write_enabled: false,
        depth_compare: CompareFunction::Always,
        stencil: StencilState::default(),
        bias: DepthBiasState::default(),
      }),
      multisample: MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
    });

    let skybox_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Skybox Vertex Buffer"),
      contents: bytemuck::cast_slice(
        &VERTICES
          .iter()
          .map(|vertex| SkyVertex {
            position: vertex.position,
          })
          .collect::<Vec<_>>(),
      ),
      usage: BufferUsages::VERTEX,
    });

    let screen = ScreenSpaceResources::construct(&device, &config);

    Ok(Self {
      last: Instant::now(),
      camera: Camera::new(),
      keys_down: HashSet::new(),
      transform,
      skybox_transform,
      surface,
      device,
      queue,
      config,
      screen,
      transform_buffer,
      transform_bind_group,
      pipeline,
      vertex_buffer,
      grass_bind_group,
      skybox_transform_buffer,
      skybox_transform_bind_group,
      skybox_pipeline,
      skybox_vertex_buffer,
    })
  }

  fn size(&self) -> PhysicalSize<u32> {
    PhysicalSize::new(self.config.width, self.config.height)
  }

  fn resize(&mut self, PhysicalSize { width, height }: PhysicalSize<u32>) {
    assert!(
      (width != 0) && (height != 0),
      "new window size had a 0 component: ({}, {})",
      width,
      height
    );

    self.config.width = width;
    self.config.height = height;

    self.surface.configure(&self.device, &self.config);

    self.screen = ScreenSpaceResources::construct(&self.device, &self.config);
  }

  fn compose(&mut self) -> Result<()> {
    let elapsed = self.last.elapsed();
    self.last = Instant::now();

    self.update(elapsed);
    self.render()?;

    Ok(())
  }

  fn press(&mut self, code: KeyCode) {
    self.keys_down.insert(code);
  }

  fn release(&mut self, code: KeyCode) {
    self.keys_down.remove(&code);
  }

  fn motion(&mut self, x: f32, y: f32) {
    const MOVEMENT_SPEED: Angle = FULL_ROTATION;

    let width = self.config.width as f32;
    let height = self.config.height as f32;

    let delta_x = x / width;
    let delta_y = y / height;

    self
      .camera
      .rotate(MOVEMENT_SPEED * delta_x, MOVEMENT_SPEED * delta_y);
  }

  fn update(&mut self, delta: Duration) {
    const CAMERA_MOVEMENT_SPEED: f32 = 10.0;

    let delta_secs = delta.as_secs_f32();

    let mut camera_movement = Vec3::default();
    for key in &self.keys_down {
      match key {
        KeyCode::KeyW => camera_movement += Z_AXIS,
        KeyCode::KeyS => camera_movement -= Z_AXIS,
        KeyCode::KeyA => camera_movement -= X_AXIS,
        KeyCode::KeyD => camera_movement += X_AXIS,
        _ => {}
      }
    }
    if camera_movement.len_sq() > 0.0 {
      self
        .camera
        .translate(CAMERA_MOVEMENT_SPEED * delta_secs * camera_movement.norm());
    }

    let PhysicalSize { width, height } = self.size();
    let world_to_screen_space = mat4::perspective(width as f32, height as f32, *FOV, Z_NEAR, Z_FAR)
      * self
        .camera
        .world_transform(if self.keys_down.contains(&KeyCode::KeyC) {
          Direction::Backward
        } else {
          Direction::Forward
        });
    self.transform = world_to_screen_space * mat4::translate(CUBE_TRANSLATE);
    self.skybox_transform = world_to_screen_space * mat4::translate(self.camera.position());
  }

  fn render(&self) -> Result<()> {
    let output = self.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&TextureViewDescriptor::default());

    self.queue.write_buffer(
      &self.skybox_transform_buffer,
      0,
      bytemuck::cast_slice(&[self.skybox_transform]),
    );
    self.queue.write_buffer(
      &self.transform_buffer,
      0,
      bytemuck::cast_slice(&[self.transform]),
    );

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });
    {
      let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color::BLACK),
            store: StoreOp::Discard,
          },
        })],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
          view: &self.screen.depth_view,
          depth_ops: Some(Operations {
            load: LoadOp::Clear(1.0),
            store: StoreOp::Discard,
          }),
          stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
      });

      render_pass.set_pipeline(&self.skybox_pipeline);
      render_pass.set_bind_group(0, &self.skybox_transform_bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.skybox_vertex_buffer.slice(..));
      render_pass.draw(0..VERTICES.len() as u32, 0..1);

      render_pass.set_pipeline(&self.pipeline);
      render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
      render_pass.set_bind_group(1, &self.grass_bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }

    self.queue.submit(iter::once(encoder.finish()));

    output.present();

    Ok(())
  }
}
