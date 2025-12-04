use crate::camera::{Camera, Direction};
use crate::core::math::aligned_box3::{AlignedBox3, BoxFace};
use crate::core::math::angle::{Angle, FULL_ROTATION};
use crate::core::math::mat4::{self, Mat4x4};
use crate::core::math::segment3::Segment3;
use crate::core::math::vec3::Vec3;
use crate::core::math::{self, X_AXIS, Y_AXIS, Z_AXIS};
use crate::platform::{Instant, ResourceReader};
use anyhow::Result;
use image::codecs::png::PngDecoder;
use image::{DynamicImage, GenericImageView};
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;
use std::{iter, mem};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
  Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
  BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
  Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages, Color,
  ColorTargetState, ColorWrites, CommandEncoderDescriptor, CompareFunction, DepthBiasState,
  DepthStencilState, Device, DeviceDescriptor, ExperimentalFeatures, Extent3d, Face, Features,
  FragmentState, FrontFace, Instance, InstanceDescriptor, Limits, LoadOp, MemoryHints,
  MultisampleState, Operations, Origin3d, PipelineCompilationOptions, PipelineLayoutDescriptor,
  PolygonMode, PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
  RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
  RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerBindingType,
  SamplerDescriptor, ShaderStages, StencilState, StoreOp, Surface, SurfaceConfiguration,
  TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension,
  TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
  TextureViewDimension, Trace, VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
  vertex_attr_array,
};
use winit::dpi::PhysicalSize;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;
use winit::window::Window;
use zerocopy::{Immutable, IntoBytes};

const BLOCK_LIMIT: usize = 256;

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
#[derive(Clone, Copy, Immutable, IntoBytes)]
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
#[derive(Clone, Copy, Immutable, IntoBytes)]
struct Quad {
  left: f32,
  right: f32,
  top: f32,
  bot: f32,
}

fn calculate_crosshair_quad(screen_width: f32, screen_height: f32, crosshair_size: u32) -> Quad {
  const WIDTH_FRACTION: f32 = 0.008;

  let size_pixels = (WIDTH_FRACTION * screen_width).ceil() as usize;
  let size_pixels = math::align(size_pixels, crosshair_size.try_into().unwrap());

  let (pixels_left, pixels_right) = math::split(size_pixels as f32);

  let (width_left, width_right) = math::split(screen_width);
  let (height_top, height_bot) = math::split(screen_height);

  Quad {
    left: -(pixels_left / width_left),
    right: pixels_right / width_right,
    // To ensure crosshair remains square, use the same amount of pixels vertically as horizontally
    top: -(pixels_left / height_top),
    bot: pixels_right / height_bot,
  }
}

const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

/// Resources that need to be constructed based on the screen's resolution, and
/// therefore reconstructed on resize.
struct ScreenSpaceResources {
  perspective: Mat4x4,
  depth_view: TextureView,
  render_view: TextureView,
  fullscreen_copy_texture_bind_group: BindGroup,
}

impl ScreenSpaceResources {
  pub fn construct(
    device: &Device,
    config: &SurfaceConfiguration,
    fullscreen_copy_texture_bind_group_layout: &BindGroupLayout,
    default_sampler: &Sampler,
  ) -> Self {
    let width = config.width;
    let height = config.height;

    let depth_texture = device.create_texture(&TextureDescriptor {
      label: Some("Depth Texture"),
      size: Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: DEPTH_FORMAT,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });

    let render_texture = device.create_texture(&TextureDescriptor {
      label: Some("Offscreen Render Texture"),
      size: Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: config.format,
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });
    let render_view = render_texture.create_view(&TextureViewDescriptor::default());

    let fullscreen_copy_texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
      label: Some("Fullscreen Copy Bind Group"),
      layout: fullscreen_copy_texture_bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::TextureView(&render_view),
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::Sampler(default_sampler),
        },
      ],
    });

    Self {
      perspective: mat4::perspective(width as f32, height as f32, *FOV, Z_NEAR, Z_FAR),
      depth_view: depth_texture.create_view(&TextureViewDescriptor::default()),
      render_view,
      fullscreen_copy_texture_bind_group,
    }
  }
}

pub struct Game {
  last: Instant,

  camera: Camera,

  keys_down: HashSet<KeyCode>,
  mouse_buttons_released: HashSet<MouseButton>,

  blocks: Vec<Vec3>,

  target_cube_index_face: Option<(usize, BoxFace)>,

  surface: Surface<'static>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  default_sampler: Sampler,

  screen: ScreenSpaceResources,

  transform_buffer: Buffer,
  transform_bind_group: BindGroup,
  pipeline: RenderPipeline,
  vertex_buffer: Buffer,
  grass_bind_group: BindGroup,

  outline_transform_buffer: Buffer,
  outline_transform_bind_group: BindGroup,
  outline_pipeline: RenderPipeline,

  skybox_transform_buffer: Buffer,
  skybox_transform_bind_group: BindGroup,
  skybox_pipeline: RenderPipeline,

  fullscreen_copy_texture_bind_group_layout: BindGroupLayout,
  fullscreen_copy_pipeline: RenderPipeline,

  crosshair_size: u32,
  crosshair_quad_buffer: Buffer,
  crosshair_bind_group: BindGroup,
  crosshair_pipeline: RenderPipeline,
}

impl Game {
  pub async fn new(window: Arc<Window>) -> Result<Self> {
    let instance = Instance::new(&InstanceDescriptor {
      backends: Backends::all(),
      ..Default::default()
    });
    let surface = instance.create_surface(Arc::clone(&window))?;
    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      })
      .await?;

    let (device, queue) = adapter
      .request_device(&DeviceDescriptor {
        required_features: Features::empty(),
        required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
        label: None,
        experimental_features: ExperimentalFeatures::disabled(),
        memory_hints: MemoryHints::Performance,
        trace: Trace::Off,
      })
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

    let default_sampler = device.create_sampler(&SamplerDescriptor::default());

    let assets = ResourceReader::new()?;

    let grass_image = assets.decode_png("textures/block/grass.png").await?;
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
      TexelCopyTextureInfo {
        texture: &grass_texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
      },
      &grass_rgba,
      TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(4 * grass_size.width),
        rows_per_image: Some(grass_size.height),
      },
      grass_size,
    );

    let grass_texture_view = grass_texture.create_view(&TextureViewDescriptor::default());
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
          resource: BindingResource::Sampler(&default_sampler),
        },
      ],
    });

    let transform_buffer = device.create_buffer(&BufferDescriptor {
      label: Some("Model -> Clip Space Transform Buffer"),
      size: (mem::size_of::<Mat4x4>() * BLOCK_LIMIT) as BufferAddress,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
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
        entry_point: Some("vs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        buffers: &[VertexBufferLayout {
          array_stride: mem::size_of::<Vertex>() as BufferAddress,
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2],
        }],
      },
      fragment: Some(FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        compilation_options: PipelineCompilationOptions::default(),
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
      cache: None,
    });

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: VERTICES.as_bytes(),
      usage: BufferUsages::VERTEX,
    });

    let outline_transform_buffer = device.create_buffer(&BufferDescriptor {
      label: Some("Model -> Clip Space Transform Buffer"),
      size: mem::size_of::<Mat4x4>() as BufferAddress,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let outline_transform_buffer_layout =
      device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
    let outline_transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
      label: Some("Transform Buffer Bind Group"),
      layout: &outline_transform_buffer_layout,
      entries: &[BindGroupEntry {
        binding: 0,
        resource: outline_transform_buffer.as_entire_binding(),
      }],
    });

    let outline_shader = device.create_shader_module(include_wgsl!("shaders/cube_outline.wgsl"));
    let outline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[&outline_transform_buffer_layout],
      push_constant_ranges: &[],
    });
    let outline_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&outline_layout),
      vertex: VertexState {
        module: &outline_shader,
        entry_point: Some("vs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        buffers: &[VertexBufferLayout {
          array_stride: mem::size_of::<Vertex>() as BufferAddress,
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x3],
        }],
      },
      fragment: Some(FragmentState {
        module: &outline_shader,
        entry_point: Some("fs_main"),
        compilation_options: PipelineCompilationOptions::default(),
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
      cache: None,
    });

    let skybox_transform_buffer = device.create_buffer(&BufferDescriptor {
      label: Some("Skybox Model -> Clip Space Transform Buffer"),
      size: mem::size_of::<Mat4x4>() as BufferAddress,
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
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
        entry_point: Some("vs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        buffers: &[VertexBufferLayout {
          array_stride: mem::size_of::<Vertex>() as BufferAddress,
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x3],
        }],
      },
      fragment: Some(FragmentState {
        module: &skybox_shader,
        entry_point: Some("fs_main"),
        compilation_options: PipelineCompilationOptions::default(),
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
      cache: None,
    });

    let fullscreen_copy_texture_bind_group_layout =
      device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Fullscreen Copy Bind Group Layout"),
        entries: &[
          BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
              sample_type: TextureSampleType::Float { filterable: false },
              view_dimension: TextureViewDimension::D2,
              multisampled: false,
            },
            count: None,
          },
          BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
            count: None,
          },
        ],
      });
    let fullscreen_copy_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Fullscreen Copy Render Pipeline Layout"),
      bind_group_layouts: &[&fullscreen_copy_texture_bind_group_layout],
      push_constant_ranges: &[],
    });
    let fullscreen_copy_shader =
      device.create_shader_module(include_wgsl!("shaders/fullscreen_copy.wgsl"));
    let fullscreen_copy_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Fullscreen Copy Render Pipeline"),
      layout: Some(&fullscreen_copy_layout),
      vertex: VertexState {
        module: &fullscreen_copy_shader,
        entry_point: Some("vs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        buffers: &[],
      },
      fragment: Some(FragmentState {
        module: &fullscreen_copy_shader,
        entry_point: Some("fs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        targets: &[Some(ColorTargetState {
          format: config.format,
          blend: Some(BlendState::REPLACE),
          write_mask: ColorWrites::ALL,
        })],
      }),
      primitive: PrimitiveState {
        topology: PrimitiveTopology::TriangleStrip,
        strip_index_format: None,
        front_face: FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: PolygonMode::Fill,
        conservative: false,
      },
      depth_stencil: None,
      multisample: MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
      cache: None,
    });

    let crosshair_image = assets.decode_png("textures/ui/crosshair.png").await?;
    let crosshair_alpha = crosshair_image.to_luma8();
    let (crosshair_width, crosshair_height) = crosshair_image.dimensions();

    let crosshair_size = Extent3d {
      width: crosshair_width,
      height: crosshair_height,
      depth_or_array_layers: 1,
    };
    let crosshair_texture = device.create_texture(&TextureDescriptor {
      label: Some("Crosshair Alpha Texture"),
      size: crosshair_size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: TextureFormat::R8Unorm,
      usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      view_formats: &[],
    });

    queue.write_texture(
      TexelCopyTextureInfo {
        texture: &crosshair_texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
      },
      &crosshair_alpha,
      TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(crosshair_size.width),
        rows_per_image: Some(crosshair_size.height),
      },
      crosshair_size,
    );

    let crosshair_quad_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Crosshair Normalised Size Buffer"),
      contents: calculate_crosshair_quad(
        config.width as f32,
        config.height as f32,
        crosshair_width,
      )
      .as_bytes(),
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let crosshair_texture_view = crosshair_texture.create_view(&TextureViewDescriptor::default());
    let crosshair_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("Crosshair Bind Group Layout"),
      entries: &[
        BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::VERTEX,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 1,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 2,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    let crosshair_bind_group = device.create_bind_group(&BindGroupDescriptor {
      label: Some("Crosshair Bind Group"),
      layout: &crosshair_bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::Buffer(crosshair_quad_buffer.as_entire_buffer_binding()),
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::TextureView(&crosshair_texture_view),
        },
        BindGroupEntry {
          binding: 2,
          resource: BindingResource::Sampler(&default_sampler),
        },
      ],
    });
    let crosshair_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Crosshair Render Pipeline Layout"),
      bind_group_layouts: &[
        &fullscreen_copy_texture_bind_group_layout,
        &crosshair_bind_group_layout,
      ],
      push_constant_ranges: &[],
    });
    let crosshair_shader = device.create_shader_module(include_wgsl!("shaders/crosshair.wgsl"));
    let crosshair_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Crosshair Render Pipeline"),
      layout: Some(&crosshair_layout),
      vertex: VertexState {
        module: &crosshair_shader,
        entry_point: Some("vs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        buffers: &[],
      },
      fragment: Some(FragmentState {
        module: &crosshair_shader,
        entry_point: Some("fs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        targets: &[Some(ColorTargetState {
          format: config.format,
          blend: Some(BlendState::ALPHA_BLENDING),
          write_mask: ColorWrites::ALL,
        })],
      }),
      primitive: PrimitiveState {
        topology: PrimitiveTopology::TriangleStrip,
        strip_index_format: None,
        front_face: FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: PolygonMode::Fill,
        conservative: false,
      },
      depth_stencil: None,
      multisample: MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
      cache: None,
    });

    let screen = ScreenSpaceResources::construct(
      &device,
      &config,
      &fullscreen_copy_texture_bind_group_layout,
      &default_sampler,
    );

    Ok(Self {
      last: Instant::now(),
      camera: Camera::new(),
      keys_down: HashSet::new(),
      mouse_buttons_released: HashSet::new(),
      blocks: Vec::from([CUBE_TRANSLATE]),
      target_cube_index_face: None,
      surface,
      device,
      queue,
      config,
      default_sampler,
      screen,
      transform_buffer,
      transform_bind_group,
      pipeline,
      vertex_buffer,
      grass_bind_group,
      outline_transform_buffer,
      outline_transform_bind_group,
      outline_pipeline,
      skybox_transform_buffer,
      skybox_transform_bind_group,
      skybox_pipeline,
      fullscreen_copy_texture_bind_group_layout,
      fullscreen_copy_pipeline,
      crosshair_size: crosshair_width,
      crosshair_quad_buffer,
      crosshair_bind_group,
      crosshair_pipeline,
    })
  }

  pub fn resize(&mut self, PhysicalSize { width, height }: PhysicalSize<u32>) {
    assert!(
      (width != 0) && (height != 0),
      "new window size had a 0 component: ({}, {})",
      width,
      height
    );

    self.config.width = width;
    self.config.height = height;

    self.surface.configure(&self.device, &self.config);

    self.screen = ScreenSpaceResources::construct(
      &self.device,
      &self.config,
      &self.fullscreen_copy_texture_bind_group_layout,
      &self.default_sampler,
    );

    self.queue.write_buffer(
      &self.crosshair_quad_buffer,
      0,
      calculate_crosshair_quad(width as f32, height as f32, self.crosshair_size).as_bytes(),
    );
  }

  pub fn compose(&mut self) -> Result<()> {
    let elapsed = self.last.elapsed();
    self.last = Instant::now();

    self.update(elapsed);
    self.render()?;

    Ok(())
  }

  pub fn press(&mut self, code: KeyCode) {
    self.keys_down.insert(code);
  }

  pub fn release(&mut self, code: KeyCode) {
    self.keys_down.remove(&code);
  }

  pub fn mouse_release(&mut self, button: MouseButton) {
    self.mouse_buttons_released.insert(button);
  }

  pub fn motion(&mut self, x: f32, y: f32) {
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
    const REACH_DISTANCE: f32 = 5.0;

    let delta_secs = delta.as_secs_f32();

    let mut camera_movement = Vec3::default();
    if self.keys_down.contains(&KeyCode::KeyW) {
      camera_movement += Z_AXIS;
    }
    if self.keys_down.contains(&KeyCode::KeyS) {
      camera_movement -= Z_AXIS;
    }
    if self.keys_down.contains(&KeyCode::KeyA) {
      camera_movement -= X_AXIS;
    }
    if self.keys_down.contains(&KeyCode::KeyD) {
      camera_movement += X_AXIS;
    }
    if self.keys_down.contains(&KeyCode::Space) {
      camera_movement += Y_AXIS;
    }
    if self.keys_down.contains(&KeyCode::ShiftLeft) {
      camera_movement -= Y_AXIS;
    }
    if camera_movement.len_sq() > 0.0 {
      self
        .camera
        .translate(CAMERA_MOVEMENT_SPEED * delta_secs * camera_movement.norm());
    }

    if let Some((index, face)) = self.target_cube_index_face {
      if self.mouse_buttons_released.contains(&MouseButton::Left) {
        self.blocks.swap_remove(index);
      } else if self.mouse_buttons_released.contains(&MouseButton::Right)
        && (self.blocks.len() < BLOCK_LIMIT)
      {
        let target_block = self.blocks.get(index).unwrap();
        let next_block = *target_block + (CUBE_SIZE * face.normal());

        self.blocks.push(next_block);
      }
    }

    let position = self.camera.position();
    let reach = Segment3::start_direction_len(position, self.camera.forward(), REACH_DISTANCE);

    self.target_cube_index_face = None;
    let mut min_dist = f32::MAX;
    for (index, block) in self.blocks.iter().enumerate() {
      if let Some(face) = AlignedBox3::cube(*block, CUBE_HALF).intersect_with(&reach) {
        let dist = (position - *block).len_sq();

        if dist < min_dist {
          self.target_cube_index_face = Some((index, face));
          min_dist = dist;
        }
      }
    }

    self.mouse_buttons_released.clear();
  }

  fn render(&self) -> Result<()> {
    let output = self.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&TextureViewDescriptor::default());

    let world_to_screen_space = &self.screen.perspective
      * &self
        .camera
        .world_transform(if self.keys_down.contains(&KeyCode::KeyC) {
          Direction::Backward
        } else {
          Direction::Forward
        });

    let skybox_transform = &world_to_screen_space * &mat4::translate(self.camera.position());
    self.queue.write_buffer(
      &self.skybox_transform_buffer,
      0,
      skybox_transform.as_bytes(),
    );

    let transforms: Vec<Mat4x4> = self
      .blocks
      .iter()
      .map(|block| &world_to_screen_space * &mat4::translate(*block))
      .collect();
    self
      .queue
      .write_buffer(&self.transform_buffer, 0, transforms.as_bytes());

    let mut encoder = self
      .device
      .create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });
    {
      let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("Main Render Pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &self.screen.render_view,
          depth_slice: None,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color::BLACK),
            store: StoreOp::Store,
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
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw(0..VERTICES.len() as u32, 0..1);

      render_pass.set_pipeline(&self.pipeline);
      render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
      render_pass.set_bind_group(1, &self.grass_bind_group, &[]);
      render_pass.draw(0..VERTICES.len() as u32, 0..self.blocks.len() as u32);

      if let Some((index, _)) = self.target_cube_index_face {
        self.queue.write_buffer(
          &self.outline_transform_buffer,
          0,
          transforms.get(index).unwrap().as_bytes(),
        );

        render_pass.set_pipeline(&self.outline_pipeline);
        render_pass.set_bind_group(0, &self.outline_transform_bind_group, &[]);
        render_pass.draw(0..VERTICES.len() as u32, 0..1);
      }
    }
    {
      let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("UI Render Pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
          view: &view,
          depth_slice: None,
          resolve_target: None,
          ops: Operations {
            load: LoadOp::Clear(Color::TRANSPARENT),
            store: StoreOp::Discard,
          },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
      });

      render_pass.set_pipeline(&self.fullscreen_copy_pipeline);
      render_pass.set_bind_group(0, &self.screen.fullscreen_copy_texture_bind_group, &[]);
      render_pass.draw(0..4, 0..1);

      render_pass.set_pipeline(&self.crosshair_pipeline);
      render_pass.set_bind_group(1, &self.crosshair_bind_group, &[]);
      render_pass.draw(0..4, 0..1);
    }

    self.queue.submit(iter::once(encoder.finish()));

    output.present();

    Ok(())
  }
}

impl ResourceReader {
  async fn decode_png(&self, path: &str) -> Result<DynamicImage> {
    let image_data = Cursor::new(self.read(path).await?);
    let decoder = PngDecoder::new(image_data)?;

    Ok(DynamicImage::from_decoder(decoder)?)
  }
}
