use anyhow::{anyhow, Result};
use bytemuck::NoUninit;
use std::time::{Duration, Instant};
use std::{iter, mem};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
  include_wgsl, vertex_attr_array, Backends, BlendState, Buffer, BufferAddress, BufferUsages,
  Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, CompareFunction, DepthBiasState,
  DepthStencilState, Device, Extent3d, Face, Features, FragmentState, FrontFace, Instance,
  InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
  PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology, Queue,
  RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
  RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, StencilState, StoreOp, Surface,
  SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
  TextureView, TextureViewDescriptor, VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

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

  let mut app = App::new(&window).await?;

  event_loop.run(|event, target| match event {
    Event::WindowEvent { event, .. } => match event {
      WindowEvent::CloseRequested
      | WindowEvent::KeyboardInput {
        event:
          KeyEvent {
            state: ElementState::Pressed,
            physical_key: PhysicalKey::Code(KeyCode::Escape),
            ..
          },
        ..
      } => {
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
      _ => {}
    },
    Event::AboutToWait => {
      window.request_redraw();
    }
    _ => {}
  })?;

  Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, NoUninit)]
struct Vertex {
  position: [f32; 3],
  color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
  Vertex {
    position: [-0.8, 0.8, 0.0],
    color: [1.0, 0.0, 0.0],
  },
  Vertex {
    position: [-0.8, -0.8, 0.0],
    color: [0.0, 1.0, 0.0],
  },
  Vertex {
    position: [0.8, 0.8, 0.0],
    color: [0.0, 0.0, 1.0],
  },
  Vertex {
    position: [0.8, 0.8, 0.0],
    color: [0.0, 0.0, 1.0],
  },
  Vertex {
    position: [-0.8, -0.8, 0.0],
    color: [0.0, 1.0, 0.0],
  },
  Vertex {
    position: [0.8, -0.8, 0.0],
    color: [1.0, 0.0, 0.0],
  },
];

struct App<'a> {
  start: Instant,

  surface: Surface<'a>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  depth_view: TextureView,
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

    let shader = device.create_shader_module(include_wgsl!("shaders/simple.wgsl"));
    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[],
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
      start: Instant::now(),
      surface,
      device,
      queue,
      config,
      depth_view,
      pipeline,
      vertex_buffer,
    })
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
    self.update(self.start.elapsed());
    self.render()?;

    Ok(())
  }

  fn update(&mut self, _delta: Duration) {}

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
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw(0..VERTICES.len() as u32, 0..1);
    }

    self.queue.submit(iter::once(encoder.finish()));

    output.present();

    Ok(())
  }
}
