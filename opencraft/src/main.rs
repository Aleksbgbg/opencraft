#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(duration_millis_float)]

mod core;

use crate::core::math::angle::{Degrees, Radians};
use crate::core::math::mat4::{self, Mat4x4};
use crate::core::math::rotor3::Rotor3;
use crate::core::math::vec3::Vec3;
use crate::core::math::{FULL_ROTATION, HALF_ROTATION, QUARTER_ROTATION, X_AXIS, Y_AXIS, Z_AXIS};
use anyhow::{anyhow, Result};
use bytemuck::NoUninit;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::{iter, mem};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
  include_wgsl, vertex_attr_array, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry,
  BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferAddress,
  BufferBindingType, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor,
  CompareFunction, DepthBiasState, DepthStencilState, Device, Extent3d, Face, Features,
  FragmentState, FrontFace, Instance, InstanceDescriptor, Limits, LoadOp, MultisampleState,
  Operations, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState,
  PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
  RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
  ShaderStages, StencilState, StoreOp, Surface, SurfaceConfiguration, TextureDescriptor,
  TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
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
  window.set_cursor_grab(CursorGrabMode::Confined)?;
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

const FOV: Degrees = Degrees::new(75.0);
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

#[repr(C)]
#[derive(Clone, Copy, NoUninit)]
struct Vertex {
  position: [f32; 3],
  color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
  // Front face
  Vertex {
    position: [LEFT, TOP, FRONT],
    color: [0.0, 1.0, 1.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, FRONT],
    color: [0.0, 0.0, 1.0],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    color: [1.0, 1.0, 1.0],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    color: [1.0, 1.0, 1.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, FRONT],
    color: [0.0, 0.0, 1.0],
  },
  Vertex {
    position: [RIGHT, BOTTOM, FRONT],
    color: [1.0, 0.0, 1.0],
  },
  // Back face
  Vertex {
    position: [LEFT, TOP, BACK],
    color: [0.0, 1.0, 0.0],
  },
  Vertex {
    position: [RIGHT, TOP, BACK],
    color: [1.0, 1.0, 0.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    color: [0.0, 0.0, 0.0],
  },
  Vertex {
    position: [RIGHT, BOTTOM, BACK],
    color: [1.0, 0.0, 0.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    color: [0.0, 0.0, 0.0],
  },
  Vertex {
    position: [RIGHT, TOP, BACK],
    color: [1.0, 1.0, 0.0],
  },
  // Top face
  Vertex {
    position: [LEFT, TOP, BACK],
    color: [0.0, 1.0, 0.0],
  },
  Vertex {
    position: [LEFT, TOP, FRONT],
    color: [0.0, 1.0, 1.0],
  },
  Vertex {
    position: [RIGHT, TOP, BACK],
    color: [1.0, 1.0, 0.0],
  },
  Vertex {
    position: [RIGHT, TOP, BACK],
    color: [1.0, 1.0, 0.0],
  },
  Vertex {
    position: [LEFT, TOP, FRONT],
    color: [0.0, 1.0, 1.0],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    color: [1.0, 1.0, 1.0],
  },
  // Bottom face
  Vertex {
    position: [RIGHT, BOTTOM, FRONT],
    color: [1.0, 0.0, 1.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, FRONT],
    color: [0.0, 0.0, 1.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    color: [0.0, 0.0, 0.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    color: [0.0, 0.0, 0.0],
  },
  Vertex {
    position: [RIGHT, BOTTOM, BACK],
    color: [1.0, 0.0, 0.0],
  },
  Vertex {
    position: [RIGHT, BOTTOM, FRONT],
    color: [1.0, 0.0, 1.0],
  },
  // Left face
  Vertex {
    position: [LEFT, TOP, BACK],
    color: [0.0, 1.0, 0.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    color: [0.0, 0.0, 0.0],
  },
  Vertex {
    position: [LEFT, TOP, FRONT],
    color: [0.0, 1.0, 1.0],
  },
  Vertex {
    position: [LEFT, TOP, FRONT],
    color: [0.0, 1.0, 1.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, BACK],
    color: [0.0, 0.0, 0.0],
  },
  Vertex {
    position: [LEFT, BOTTOM, FRONT],
    color: [0.0, 0.0, 1.0],
  },
  // Right face
  Vertex {
    position: [RIGHT, TOP, BACK],
    color: [1.0, 1.0, 0.0],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    color: [1.0, 1.0, 1.0],
  },
  Vertex {
    position: [RIGHT, BOTTOM, BACK],
    color: [1.0, 0.0, 0.0],
  },
  Vertex {
    position: [RIGHT, BOTTOM, BACK],
    color: [1.0, 0.0, 0.0],
  },
  Vertex {
    position: [RIGHT, TOP, FRONT],
    color: [1.0, 1.0, 1.0],
  },
  Vertex {
    position: [RIGHT, BOTTOM, FRONT],
    color: [1.0, 0.0, 1.0],
  },
];

#[derive(Default)]
struct Camera {
  position: Vec3,
  rotation_x: Radians,
  rotation_y: Radians,
}

impl Camera {
  pub fn new() -> Self {
    Self::default()
  }

  fn rotor(rotation_x: Radians, rotation_y: Radians) -> Rotor3 {
    let rotor_x = {
      let orientation_x = Z_AXIS.angle_axis_rotate(rotation_x, X_AXIS);
      Rotor3::new(Z_AXIS, orientation_x)
    };
    let rotor_y = if rotation_y == HALF_ROTATION {
      let midpoint = Z_AXIS.angle_axis_rotate(QUARTER_ROTATION, Y_AXIS);
      Rotor3::new(Z_AXIS, midpoint) * Rotor3::new(midpoint, -Z_AXIS)
    } else {
      let orientation_y = Z_AXIS.angle_axis_rotate(rotation_y, Y_AXIS);
      Rotor3::new(Z_AXIS, orientation_y)
    };

    rotor_x * rotor_y
  }

  pub fn translate(&mut self, offset: Vec3) {
    self.position += Self::rotor(self.rotation_x, self.rotation_y).rotate(offset);
  }

  pub fn rotate<X: Into<Radians>, Y: Into<Radians>>(&mut self, x: X, y: Y) {
    self.rotation_x += x.into();
    self.rotation_y += y.into();

    self.rotation_x = self.rotation_x.clamp();
    self.rotation_y = self.rotation_y.clamp();
  }

  /// Returns a transformation to be applied on the world to simulate the
  /// position of the camera. The world transformation will be the inverse of
  /// all movements applied on the camera, as (for example) moving the camera
  /// backwards can be simulated by moving the entire world forwards.
  pub fn world_transform(&self, reverse: bool) -> Mat4x4 {
    let rotation_y = if reverse {
      self.rotation_y + HALF_ROTATION
    } else {
      self.rotation_y
    };
    mat4::rotate(-Self::rotor(self.rotation_x, rotation_y)) * mat4::translate(-self.position)
  }
}

struct App<'a> {
  last: Instant,

  camera: Camera,
  keys_down: HashSet<KeyCode>,
  rotation: Degrees,
  transform: Mat4x4,

  surface: Surface<'a>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  depth_view: TextureView,
  transform_buffer: Buffer,
  transform_bind_group: BindGroup,
  pipeline: RenderPipeline,
  vertex_buffer: Buffer,
}

const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;
fn create_depth_texture(device: &Device, config: &SurfaceConfiguration) -> TextureView {
  let texture = device.create_texture(&TextureDescriptor {
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

  texture.create_view(&TextureViewDescriptor::default())
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
      present_mode: capabilities.present_modes[0],
      alpha_mode: capabilities.alpha_modes[0],
      view_formats: Vec::new(),
      desired_maximum_frame_latency: 3,
    };

    surface.configure(&device, &config);

    let depth_view = create_depth_texture(&device, &config);

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

    let shader = device.create_shader_module(include_wgsl!("shaders/simple.wgsl"));
    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[&transform_buffer_layout],
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

    Ok(Self {
      last: Instant::now(),
      camera: Camera::new(),
      keys_down: HashSet::new(),
      rotation: Degrees::new(0.0),
      transform,
      surface,
      device,
      queue,
      config,
      depth_view,
      transform_buffer,
      transform_bind_group,
      pipeline,
      vertex_buffer,
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

    self.depth_view = create_depth_texture(&self.device, &self.config);
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
    const MOVEMENT_SPEED: Radians = FULL_ROTATION;

    let width = self.config.width as f32;
    let height = self.config.height as f32;

    let delta_x = x / width;
    let delta_y = y / height;

    // Swap axes as we want the mouse movement across the screen (x) to rotate
    // across the Y axis and vice-versa.
    self
      .camera
      .rotate(MOVEMENT_SPEED * delta_y, MOVEMENT_SPEED * delta_x);
  }

  fn update(&mut self, delta: Duration) {
    const CUBE_ROTATION_SPEED: f32 = 100.0;
    const CAMERA_MOVEMENT_SPEED: f32 = 10.0;

    let delta_secs = delta.as_secs_f32();

    self.rotation += Degrees::new(CUBE_ROTATION_SPEED * delta_secs);
    self.rotation = self.rotation.clamp();

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
    self.transform = mat4::perspective(width as f32, height as f32, FOV, Z_NEAR, Z_FAR)
      * self
        .camera
        .world_transform(self.keys_down.contains(&KeyCode::KeyC))
      * mat4::translate(CUBE_TRANSLATE)
      * mat4::rotate({
        let a = (X_AXIS + Y_AXIS + Z_AXIS).norm();
        let b = a.angle_axis_rotate(self.rotation, a.perpendicular());

        Rotor3::new(a, b)
      })
  }

  fn render(&self) -> Result<()> {
    let output = self.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&TextureViewDescriptor::default());

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
            load: LoadOp::Clear(Color {
              r: 0.1,
              g: 0.2,
              b: 0.3,
              a: 1.0,
            }),
            store: StoreOp::Store,
          },
        })],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
          view: &self.depth_view,
          depth_ops: Some(Operations {
            load: LoadOp::Clear(1.0),
            store: StoreOp::Store,
          }),
          stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
      });
      render_pass.set_pipeline(&self.pipeline);
      render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }

    self.queue.write_buffer(
      &self.transform_buffer,
      0,
      bytemuck::cast_slice(&[self.transform]),
    );
    self.queue.submit(iter::once(encoder.finish()));

    output.present();

    Ok(())
  }
}
