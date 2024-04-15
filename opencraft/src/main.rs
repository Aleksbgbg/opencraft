use anyhow::{anyhow, Result};
use std::iter;
use wgpu::{
  Backends, Color, CommandEncoderDescriptor, Device, Features, Instance, InstanceDescriptor,
  Limits, LoadOp, Operations, PowerPreference, Queue, RenderPassColorAttachment,
  RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration,
  TextureFormat, TextureUsages, TextureViewDescriptor,
};
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

  let app = App::new(&window).await?;

  #[allow(clippy::single_match)]
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
        if let Err(err) = app.render() {
          eprintln!("Error during render: {:?}", err);
          target.exit();
        }
      }
      _ => {}
    },
    _ => {}
  })?;

  Ok(())
}

struct App<'a> {
  surface: Surface<'a>,
  device: Device,
  queue: Queue,
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

    Ok(Self {
      surface,
      device,
      queue,
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
    encoder.begin_render_pass(&RenderPassDescriptor {
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
      depth_stencil_attachment: None,
      occlusion_query_set: None,
      timestamp_writes: None,
    });
    self.queue.submit(iter::once(encoder.finish()));

    output.present();

    Ok(())
  }
}
