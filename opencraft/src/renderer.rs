pub mod chunk_mesh;
mod display;
mod font_atlas;
mod text_encoder;

use crate::camera::Direction;
use crate::core::math;
use crate::core::math::angle::Angle;
use crate::core::math::mat4;
use crate::core::math::mat4::Mat4x4;
use crate::core::math::vec2::Vec2;
use crate::core::poll_on_interval::PollOnInterval;
use crate::core::type_conversions::{Coerce, CoerceLossy, CoerceLossyCeil};
use crate::model::Scene;
use crate::model::chunk::Chunk;
use crate::model::layout::CUBE_EXTENT;
use crate::model::position::{BlockPosition, ChunkPosition};
use crate::platform::ResourceReader;
use crate::renderer::chunk_mesh::ChunkMesh;
use crate::renderer::display::Bytes;
use crate::renderer::font_atlas::{FontAtlas, TextVertex};
use crate::renderer::text_encoder::{Anchor, EMPTY_LINE, TextEncoder};
use crate::resources::Texture;
use crate::{core, platform};
use anyhow::Result;
use image::GenericImageView;
use memory_stats::MemoryStats;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use std::{iter, mem};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::TextureDataOrder;
use wgpu::{
  Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
  BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
  Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites,
  CommandEncoderDescriptor, CompareFunction, DepthBiasState, DepthStencilState, Device,
  DeviceDescriptor, ExperimentalFeatures, Extent3d, Face, Features, FragmentState, FrontFace,
  Instance, InstanceDescriptor, Limits, LoadOp, MemoryHints, MultisampleState, Operations,
  PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PresentMode,
  PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment,
  RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
  RequestAdapterOptions, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
  StencilState, StoreOp, Surface, SurfaceConfiguration, TextureDescriptor, TextureDimension,
  TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
  TextureViewDimension, Trace, VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
  vertex_attr_array,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use zerocopy::{Immutable, IntoBytes};

const FONT_SCALE: f32 = 24.0;

static HORIZONTAL_FOV: LazyLock<Angle> = LazyLock::new(|| Angle::degrees(110.0));
const Z_NEAR: f32 = 0.01;
const Z_FAR: f32 = 1000.0;

const X_POS: f32 = CUBE_EXTENT;
const X_NEG: f32 = -CUBE_EXTENT;
const Y_POS: f32 = CUBE_EXTENT;
const Y_NEG: f32 = -CUBE_EXTENT;
const Z_POS: f32 = CUBE_EXTENT;
const Z_NEG: f32 = -CUBE_EXTENT;

const TEX_WIDTH: f32 = 48.0;
const TEX_HEIGHT: f32 = 64.0;

const TEX_X_POS_LEFT: f32 = 32.0 / TEX_WIDTH;
const TEX_X_POS_RIGHT: f32 = 48.0 / TEX_WIDTH;
const TEX_X_POS_TOP: f32 = 16.0 / TEX_HEIGHT;
const TEX_X_POS_BOTTOM: f32 = 32.0 / TEX_HEIGHT;

const TEX_X_NEG_LEFT: f32 = 0.0 / TEX_WIDTH;
const TEX_X_NEG_RIGHT: f32 = 16.0 / TEX_WIDTH;
const TEX_X_NEG_TOP: f32 = 16.0 / TEX_HEIGHT;
const TEX_X_NEG_BOTTOM: f32 = 32.0 / TEX_HEIGHT;

const TEX_Y_POS_LEFT: f32 = 16.0 / TEX_WIDTH;
const TEX_Y_POS_RIGHT: f32 = 32.0 / TEX_WIDTH;
const TEX_Y_POS_TOP: f32 = 16.0 / TEX_HEIGHT;
const TEX_Y_POS_BOTTOM: f32 = 32.0 / TEX_HEIGHT;

const TEX_Y_NEG_LEFT: f32 = 16.0 / TEX_WIDTH;
const TEX_Y_NEG_RIGHT: f32 = 32.0 / TEX_WIDTH;
const TEX_Y_NEG_TOP: f32 = 48.0 / TEX_HEIGHT;
const TEX_Y_NEG_BOTTOM: f32 = 64.0 / TEX_HEIGHT;

const TEX_Z_POS_LEFT: f32 = 16.0 / TEX_WIDTH;
const TEX_Z_POS_RIGHT: f32 = 32.0 / TEX_WIDTH;
const TEX_Z_POS_TOP: f32 = 0.0 / TEX_HEIGHT;
const TEX_Z_POS_BOTTOM: f32 = 16.0 / TEX_HEIGHT;

const TEX_Z_NEG_LEFT: f32 = 16.0 / TEX_WIDTH;
const TEX_Z_NEG_RIGHT: f32 = 32.0 / TEX_WIDTH;
const TEX_Z_NEG_TOP: f32 = 32.0 / TEX_HEIGHT;
const TEX_Z_NEG_BOTTOM: f32 = 48.0 / TEX_HEIGHT;

#[repr(C)]
#[derive(Clone, Copy, Immutable, IntoBytes)]
pub struct Vertex {
  position: [f32; 3],
  texture_coordinate: [f32; 2],
}

pub const VERTICES: &[Vertex] = &[
  // +X face
  Vertex {
    position: [X_POS, Y_POS, Z_NEG],
    texture_coordinate: [TEX_X_POS_LEFT, TEX_X_POS_BOTTOM],
  },
  Vertex {
    position: [X_POS, Y_POS, Z_POS],
    texture_coordinate: [TEX_X_POS_LEFT, TEX_X_POS_TOP],
  },
  Vertex {
    position: [X_POS, Y_NEG, Z_NEG],
    texture_coordinate: [TEX_X_POS_RIGHT, TEX_X_POS_BOTTOM],
  },
  Vertex {
    position: [X_POS, Y_NEG, Z_NEG],
    texture_coordinate: [TEX_X_POS_RIGHT, TEX_X_POS_BOTTOM],
  },
  Vertex {
    position: [X_POS, Y_POS, Z_POS],
    texture_coordinate: [TEX_X_POS_LEFT, TEX_X_POS_TOP],
  },
  Vertex {
    position: [X_POS, Y_NEG, Z_POS],
    texture_coordinate: [TEX_X_POS_RIGHT, TEX_X_POS_TOP],
  },
  // -X face
  Vertex {
    position: [X_NEG, Y_POS, Z_POS],
    texture_coordinate: [TEX_X_NEG_RIGHT, TEX_X_NEG_TOP],
  },
  Vertex {
    position: [X_NEG, Y_POS, Z_NEG],
    texture_coordinate: [TEX_X_NEG_RIGHT, TEX_X_NEG_BOTTOM],
  },
  Vertex {
    position: [X_NEG, Y_NEG, Z_POS],
    texture_coordinate: [TEX_X_NEG_LEFT, TEX_X_NEG_TOP],
  },
  Vertex {
    position: [X_NEG, Y_NEG, Z_POS],
    texture_coordinate: [TEX_X_NEG_LEFT, TEX_X_NEG_TOP],
  },
  Vertex {
    position: [X_NEG, Y_POS, Z_NEG],
    texture_coordinate: [TEX_X_NEG_RIGHT, TEX_X_NEG_BOTTOM],
  },
  Vertex {
    position: [X_NEG, Y_NEG, Z_NEG],
    texture_coordinate: [TEX_X_NEG_LEFT, TEX_X_NEG_BOTTOM],
  },
  // +Y face
  Vertex {
    position: [X_POS, Y_POS, Z_NEG],
    texture_coordinate: [TEX_Y_POS_LEFT, TEX_Y_POS_TOP],
  },
  Vertex {
    position: [X_NEG, Y_POS, Z_NEG],
    texture_coordinate: [TEX_Y_POS_RIGHT, TEX_Y_POS_TOP],
  },
  Vertex {
    position: [X_POS, Y_POS, Z_POS],
    texture_coordinate: [TEX_Y_POS_LEFT, TEX_Y_POS_BOTTOM],
  },
  Vertex {
    position: [X_POS, Y_POS, Z_POS],
    texture_coordinate: [TEX_Y_POS_LEFT, TEX_Y_POS_BOTTOM],
  },
  Vertex {
    position: [X_NEG, Y_POS, Z_NEG],
    texture_coordinate: [TEX_Y_POS_RIGHT, TEX_Y_POS_TOP],
  },
  Vertex {
    position: [X_NEG, Y_POS, Z_POS],
    texture_coordinate: [TEX_Y_POS_RIGHT, TEX_Y_POS_BOTTOM],
  },
  // -Y face
  Vertex {
    position: [X_POS, Y_NEG, Z_POS],
    texture_coordinate: [TEX_Y_NEG_LEFT, TEX_Y_NEG_TOP],
  },
  Vertex {
    position: [X_NEG, Y_NEG, Z_POS],
    texture_coordinate: [TEX_Y_NEG_RIGHT, TEX_Y_NEG_TOP],
  },
  Vertex {
    position: [X_POS, Y_NEG, Z_NEG],
    texture_coordinate: [TEX_Y_NEG_LEFT, TEX_Y_NEG_BOTTOM],
  },
  Vertex {
    position: [X_POS, Y_NEG, Z_NEG],
    texture_coordinate: [TEX_Y_NEG_LEFT, TEX_Y_NEG_BOTTOM],
  },
  Vertex {
    position: [X_NEG, Y_NEG, Z_POS],
    texture_coordinate: [TEX_Y_NEG_RIGHT, TEX_Y_NEG_TOP],
  },
  Vertex {
    position: [X_NEG, Y_NEG, Z_NEG],
    texture_coordinate: [TEX_Y_NEG_RIGHT, TEX_Y_NEG_BOTTOM],
  },
  // +Z face
  Vertex {
    position: [X_POS, Y_POS, Z_POS],
    texture_coordinate: [TEX_Z_POS_RIGHT, TEX_Z_POS_BOTTOM],
  },
  Vertex {
    position: [X_NEG, Y_POS, Z_POS],
    texture_coordinate: [TEX_Z_POS_LEFT, TEX_Z_POS_BOTTOM],
  },
  Vertex {
    position: [X_POS, Y_NEG, Z_POS],
    texture_coordinate: [TEX_Z_POS_RIGHT, TEX_Z_POS_TOP],
  },
  Vertex {
    position: [X_POS, Y_NEG, Z_POS],
    texture_coordinate: [TEX_Z_POS_RIGHT, TEX_Z_POS_TOP],
  },
  Vertex {
    position: [X_NEG, Y_POS, Z_POS],
    texture_coordinate: [TEX_Z_POS_LEFT, TEX_Z_POS_BOTTOM],
  },
  Vertex {
    position: [X_NEG, Y_NEG, Z_POS],
    texture_coordinate: [TEX_Z_POS_LEFT, TEX_Z_POS_TOP],
  },
  // -Z face
  Vertex {
    position: [X_NEG, Y_POS, Z_NEG],
    texture_coordinate: [TEX_Z_NEG_LEFT, TEX_Z_NEG_TOP],
  },
  Vertex {
    position: [X_POS, Y_POS, Z_NEG],
    texture_coordinate: [TEX_Z_NEG_RIGHT, TEX_Z_NEG_TOP],
  },
  Vertex {
    position: [X_NEG, Y_NEG, Z_NEG],
    texture_coordinate: [TEX_Z_NEG_LEFT, TEX_Z_NEG_BOTTOM],
  },
  Vertex {
    position: [X_NEG, Y_NEG, Z_NEG],
    texture_coordinate: [TEX_Z_NEG_LEFT, TEX_Z_NEG_BOTTOM],
  },
  Vertex {
    position: [X_POS, Y_POS, Z_NEG],
    texture_coordinate: [TEX_Z_NEG_RIGHT, TEX_Z_NEG_TOP],
  },
  Vertex {
    position: [X_POS, Y_NEG, Z_NEG],
    texture_coordinate: [TEX_Z_NEG_RIGHT, TEX_Z_NEG_BOTTOM],
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

fn calculate_crosshair_quad(screen_size: Vec2, crosshair_size: u32) -> Quad {
  const WIDTH_FRACTION: f32 = 0.008;

  let size_pixels = (WIDTH_FRACTION * screen_size.x()).coerce_lossy_ceil();
  let size_pixels = math::align(size_pixels, crosshair_size.coerce());

  let (pixels_left, pixels_right) = math::split(size_pixels.coerce_lossy());

  let (width_left, width_right) = math::split(screen_size.x());
  let (height_top, height_bot) = math::split(screen_size.y());

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
      perspective: mat4::perspective(
        width.coerce_lossy(),
        height.coerce_lossy(),
        *HORIZONTAL_FOV,
        Z_NEAR,
        Z_FAR,
      ),
      depth_view: depth_texture.create_view(&TextureViewDescriptor::default()),
      render_view,
      fullscreen_copy_texture_bind_group,
    }
  }
}

pub struct Renderer {
  graphics_backend_string: &'static str,
  memory_stats: PollOnInterval<Option<MemoryStats>>,

  font_atlas: FontAtlas,

  surface: Surface<'static>,
  device: Device,
  queue: Queue,
  config: SurfaceConfiguration,
  default_sampler: Sampler,

  screen: ScreenSpaceResources,

  vertex_buffer: Buffer,

  block_world_to_screen_transform_buffer: Buffer,
  block_world_to_screen_transform_bind_group: BindGroup,
  block_pipeline: RenderPipeline,
  grass_bind_group: BindGroup,

  chunks: HashMap<ChunkPosition, ChunkRender>,

  block_outline_transform_buffer: Buffer,
  block_outline_transform_bind_group: BindGroup,
  block_outline_pipeline: RenderPipeline,

  skybox_transform_buffer: Buffer,
  skybox_transform_bind_group: BindGroup,
  skybox_pipeline: RenderPipeline,

  fullscreen_copy_texture_bind_group_layout: BindGroupLayout,
  fullscreen_copy_pipeline: RenderPipeline,

  crosshair_size: u32,
  crosshair_quad_buffer: Buffer,
  crosshair_bind_group: BindGroup,
  crosshair_pipeline: RenderPipeline,

  text_buffer: Option<Buffer>,
  text_bind_group: BindGroup,
  text_pipeline: RenderPipeline,
}

impl Renderer {
  pub async fn new(window: Arc<Window>) -> Result<Self> {
    let instance = Instance::new(&InstanceDescriptor {
      backends: Backends::all(),
      ..Default::default()
    });
    let surface = instance.create_surface(Arc::clone(&window))?;
    let adapter = instance
      .request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
      })
      .await?;

    let (device, queue) = adapter
      .request_device(&DeviceDescriptor {
        label: None,
        required_features: Features::empty(),
        required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
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
      desired_maximum_frame_latency: 3,
      alpha_mode: capabilities.alpha_modes[0],
      view_formats: Vec::new(),
    };

    surface.configure(&device, &config);

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: VERTICES.as_bytes(),
      usage: BufferUsages::VERTEX,
    });

    let default_sampler = device.create_sampler(&SamplerDescriptor::default());

    let assets = ResourceReader::new()?;

    let grass_image = assets.load_texture(Texture::Grass).await?;
    let grass_rgba = grass_image.to_rgba8();
    let (grass_width, grass_height) = grass_image.dimensions();

    let grass_texture = device.create_texture_with_data(
      &queue,
      &TextureDescriptor {
        label: Some("Grass Texture"),
        size: Extent3d {
          width: grass_width,
          height: grass_height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      },
      TextureDataOrder::default(),
      &grass_rgba,
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

    let block_world_to_screen_transform_buffer = device.create_buffer(&BufferDescriptor {
      label: Some("Block World -> Screen Transform Buffer"),
      size: mem::size_of::<Mat4x4>().coerce(),
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let block_world_to_screen_transform_layout =
      device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Block World -> Screen Transform Buffer Bind Group Layout"),
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
    let block_world_to_screen_transform_bind_group =
      device.create_bind_group(&BindGroupDescriptor {
        label: Some("Block World -> Screen Transform Buffer Bind Group"),
        layout: &block_world_to_screen_transform_layout,
        entries: &[BindGroupEntry {
          binding: 0,
          resource: block_world_to_screen_transform_buffer.as_entire_binding(),
        }],
      });

    let block_shader = device.create_shader_module(include_wgsl!("shaders/block.wgsl"));
    let block_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Block Render Pipeline Layout"),
      bind_group_layouts: &[
        &block_world_to_screen_transform_layout,
        &grass_bind_group_layout,
      ],
      immediate_size: 0,
    });
    let block_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Block Render Pipeline"),
      layout: Some(&block_layout),
      vertex: VertexState {
        module: &block_shader,
        entry_point: Some("vs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        buffers: &[VertexBufferLayout {
          array_stride: mem::size_of::<Vertex>().coerce(),
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x3, 1 => Float32x2],
        }],
      },
      fragment: Some(FragmentState {
        module: &block_shader,
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
      multiview_mask: None,
      cache: None,
    });

    let block_outline_transform_buffer = device.create_buffer(&BufferDescriptor {
      label: Some("Block Outline Model -> Clip Space Transform Buffer"),
      size: mem::size_of::<Mat4x4>().coerce(),
      usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let block_outline_transform_buffer_layout =
      device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("Block Outline Transform Buffer Bind Group Layout"),
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
    let block_outline_transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
      label: Some("Block Outline Transform Buffer Bind Group"),
      layout: &block_outline_transform_buffer_layout,
      entries: &[BindGroupEntry {
        binding: 0,
        resource: block_outline_transform_buffer.as_entire_binding(),
      }],
    });

    let block_outline_shader =
      device.create_shader_module(include_wgsl!("shaders/block_outline.wgsl"));
    let block_outline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Block Outline Render Pipeline Layout"),
      bind_group_layouts: &[&block_outline_transform_buffer_layout],
      immediate_size: 0,
    });
    let block_outline_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Block Outline Render Pipeline"),
      layout: Some(&block_outline_layout),
      vertex: VertexState {
        module: &block_outline_shader,
        entry_point: Some("vs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        buffers: &[VertexBufferLayout {
          array_stride: mem::size_of::<Vertex>().coerce(),
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x3],
        }],
      },
      fragment: Some(FragmentState {
        module: &block_outline_shader,
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
      multiview_mask: None,
      cache: None,
    });

    let skybox_transform_buffer = device.create_buffer(&BufferDescriptor {
      label: Some("Skybox Model -> Clip Space Transform Buffer"),
      size: mem::size_of::<Mat4x4>().coerce(),
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
      immediate_size: 0,
    });
    let skybox_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Skybox Render Pipeline"),
      layout: Some(&skybox_layout),
      vertex: VertexState {
        module: &skybox_shader,
        entry_point: Some("vs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        buffers: &[VertexBufferLayout {
          array_stride: mem::size_of::<Vertex>().coerce(),
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
        front_face: FrontFace::Ccw,
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
      multiview_mask: None,
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
      immediate_size: 0,
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
        front_face: FrontFace::Cw,
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
      multiview_mask: None,
      cache: None,
    });

    let crosshair_image = assets.load_texture(Texture::Crosshair).await?;
    let crosshair_alpha = crosshair_image.to_luma8();
    let (crosshair_width, crosshair_height) = crosshair_image.dimensions();

    let crosshair_texture = device.create_texture_with_data(
      &queue,
      &TextureDescriptor {
        label: Some("Crosshair Alpha Texture"),
        size: Extent3d {
          width: crosshair_width,
          height: crosshair_height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::R8Unorm,
        usage: TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      },
      TextureDataOrder::default(),
      &crosshair_alpha,
    );

    let crosshair_quad_buffer = device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Crosshair Normalised Size Buffer"),
      contents: calculate_crosshair_quad(
        Vec2::new(config.width.coerce_lossy(), config.height.coerce_lossy()),
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
      immediate_size: 0,
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
        front_face: FrontFace::Cw,
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
      multiview_mask: None,
      cache: None,
    });

    let (font_atlas, font_atlas_alpha) = FontAtlas::load(&assets, FONT_SCALE).await?;
    let (font_atlas_width, font_atlas_height) = font_atlas.dimensions();

    let font_atlas_texture = device.create_texture_with_data(
      &queue,
      &TextureDescriptor {
        label: Some("Font Atlas Alpha Texture"),
        size: Extent3d {
          width: font_atlas_width,
          height: font_atlas_height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::R8Unorm,
        usage: TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      },
      TextureDataOrder::default(),
      &font_atlas_alpha,
    );

    let font_atlas_view = font_atlas_texture.create_view(&TextureViewDescriptor::default());
    let text_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: Some("Text Bind Group Layout"),
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
    let text_bind_group = device.create_bind_group(&BindGroupDescriptor {
      label: Some("Text Bind Group"),
      layout: &text_bind_group_layout,
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::TextureView(&font_atlas_view),
        },
        BindGroupEntry {
          binding: 1,
          resource: BindingResource::Sampler(&default_sampler),
        },
      ],
    });
    let text_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
      label: Some("Text Render Pipeline Layout"),
      bind_group_layouts: &[&text_bind_group_layout],
      immediate_size: 0,
    });
    let text_shader = device.create_shader_module(include_wgsl!("shaders/text.wgsl"));
    let text_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Text Render Pipeline"),
      layout: Some(&text_layout),
      vertex: VertexState {
        module: &text_shader,
        entry_point: Some("vs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        buffers: &[VertexBufferLayout {
          array_stride: mem::size_of::<TextVertex>().coerce(),
          step_mode: VertexStepMode::Vertex,
          attributes: &vertex_attr_array![0 => Float32x2, 1 => Float32x2],
        }],
      },
      fragment: Some(FragmentState {
        module: &text_shader,
        entry_point: Some("fs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        targets: &[Some(ColorTargetState {
          format: config.format,
          blend: Some(BlendState::ALPHA_BLENDING),
          write_mask: ColorWrites::ALL,
        })],
      }),
      primitive: PrimitiveState {
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: FrontFace::Cw,
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
      multiview_mask: None,
      cache: None,
    });

    let screen = ScreenSpaceResources::construct(
      &device,
      &config,
      &fullscreen_copy_texture_bind_group_layout,
      &default_sampler,
    );

    Ok(Self {
      graphics_backend_string: platform::get_graphics_backend_string(adapter.get_info().backend),
      memory_stats: PollOnInterval::new(memory_stats::memory_stats, Duration::from_secs(2)),
      font_atlas,
      surface,
      device,
      queue,
      config,
      default_sampler,
      screen,
      vertex_buffer,
      block_world_to_screen_transform_buffer,
      block_world_to_screen_transform_bind_group,
      block_pipeline,
      grass_bind_group,
      chunks: HashMap::new(),
      block_outline_transform_buffer,
      block_outline_transform_bind_group,
      block_outline_pipeline,
      skybox_transform_buffer,
      skybox_transform_bind_group,
      skybox_pipeline,
      fullscreen_copy_texture_bind_group_layout,
      fullscreen_copy_pipeline,
      crosshair_size: crosshair_width,
      crosshair_quad_buffer,
      crosshair_bind_group,
      crosshair_pipeline,
      text_buffer: None,
      text_bind_group,
      text_pipeline,
    })
  }

  pub fn screen_size(&self) -> Vec2 {
    Vec2::new(
      self.config.width.coerce_lossy(),
      self.config.height.coerce_lossy(),
    )
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
      calculate_crosshair_quad(
        Vec2::new(width.coerce_lossy(), height.coerce_lossy()),
        self.crosshair_size,
      )
      .as_bytes(),
    );
  }

  pub fn render(&mut self, scene: &Scene<'_>, view_direction: Direction) -> Result<()> {
    for position in scene.unloaded_chunks {
      self.chunks.remove(position);
    }
    for (position, chunk) in scene
      .loaded_chunks
      .iter()
      .filter_map(|position| scene.chunks.get(position).map(|chunk| (*position, chunk)))
    {
      self.chunks.insert(
        position,
        ChunkRender::load(
          &ChunkGraphicsResources {
            device: &self.device,
          },
          chunk,
        ),
      );
    }

    for &(chunk_position, block_position) in scene.destroyed_blocks {
      let chunk_render = self.chunks.get_mut(&chunk_position).unwrap();
      chunk_render.destroy_block(
        &ChunkGraphicsResources {
          device: &self.device,
        },
        scene.chunks.get(&chunk_position).unwrap(),
        block_position,
      );
    }
    for &(chunk_position, block_position) in scene.created_blocks {
      let chunk_render = self.chunks.get_mut(&chunk_position).unwrap();
      chunk_render.create_block(
        &ChunkGraphicsResources {
          device: &self.device,
        },
        scene.chunks.get(&chunk_position).unwrap(),
        block_position,
      );
    }

    let output = self.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&TextureViewDescriptor::default());

    let world_to_screen_space =
      &self.screen.perspective * &scene.player_camera.world_transform(view_direction);
    self.queue.write_buffer(
      &self.block_world_to_screen_transform_buffer,
      0,
      world_to_screen_space.as_bytes(),
    );

    let skybox_transform =
      &world_to_screen_space * &mat4::translate(scene.player_camera.position());
    self.queue.write_buffer(
      &self.skybox_transform_buffer,
      0,
      skybox_transform.as_bytes(),
    );

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
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
      });

      render_pass.set_pipeline(&self.skybox_pipeline);
      render_pass.set_bind_group(0, &self.skybox_transform_bind_group, &[]);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw(0..VERTICES.len().coerce(), 0..1);

      if let Some(target_block) = &scene.target_block {
        self.queue.write_buffer(
          &self.block_outline_transform_buffer,
          0,
          (&world_to_screen_space * &mat4::translate(target_block.world_position)).as_bytes(),
        );

        render_pass.set_pipeline(&self.block_outline_pipeline);
        render_pass.set_bind_group(0, &self.block_outline_transform_bind_group, &[]);
        render_pass.draw(0..VERTICES.len().coerce(), 0..1);
      }

      render_pass.set_pipeline(&self.block_pipeline);
      render_pass.set_bind_group(0, &self.block_world_to_screen_transform_bind_group, &[]);
      render_pass.set_bind_group(1, &self.grass_bind_group, &[]);
      for chunk in self.chunks.values() {
        render_pass.set_vertex_buffer(0, chunk.vertex_buffer.slice(..));
        render_pass.draw(0..chunk.vertices_len, 0..1);
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
            store: StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
      });

      render_pass.set_pipeline(&self.fullscreen_copy_pipeline);
      render_pass.set_bind_group(0, &self.screen.fullscreen_copy_texture_bind_group, &[]);
      render_pass.draw(0..4, 0..1);

      render_pass.set_pipeline(&self.crosshair_pipeline);
      render_pass.set_bind_group(1, &self.crosshair_bind_group, &[]);
      render_pass.draw(0..4, 0..1);

      if let Some(debug_display) = &scene.debug_display {
        let mut text_encoder = TextEncoder::new(
          &self.font_atlas,
          PhysicalSize::new(self.config.width, self.config.height),
        );

        text_encoder.push_text_block(
          &[&format!(
            "FPS: {} ({:.3}ms)",
            debug_display.frames_per_second, debug_display.mean_frame_time_ms
          )],
          Anchor {
            left: Some(5),
            top: Some(5),
            ..Default::default()
          },
        );

        if let Some(usage) = self.memory_stats.poll() {
          text_encoder.push_text_block(
            &[
              self.graphics_backend_string,
              EMPTY_LINE,
              &format!("RAM: {}", Bytes(usage.physical_mem)),
            ],
            Anchor {
              right: Some(5),
              top: Some(5),
              ..Default::default()
            },
          );
        } else {
          text_encoder.push_text_block(
            &[self.graphics_backend_string],
            Anchor {
              right: Some(5),
              top: Some(5),
              ..Default::default()
            },
          );
        }

        if let Some(target_block) = &scene.target_block {
          text_encoder.push_text_block(
            &[
              &format!("Looking at: {:?}", target_block.block),
              &format!("  Chunk: {}", target_block.chunk_position),
              &format!(
                "  Block: {} [{}]",
                target_block.block_position, target_block.face,
              ),
              &format!("  World: {}", target_block.world_position),
            ],
            Anchor {
              left: Some(5),
              bottom: Some(5),
              ..Default::default()
            },
          );
        }

        let text_vertices = text_encoder.finish();

        if let Some(text_buffer) = &self.text_buffer {
          if text_buffer.size() < core::slice_byte_len(&text_vertices).coerce() {
            self.create_text_buffer(&text_vertices);
          } else {
            self
              .queue
              .write_buffer(text_buffer, 0, text_vertices.as_bytes());
          }
        } else {
          self.create_text_buffer(&text_vertices);
        }

        render_pass.set_pipeline(&self.text_pipeline);
        render_pass.set_bind_group(0, &self.text_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.text_buffer.as_ref().unwrap().slice(..));
        render_pass.draw(0..text_vertices.len().coerce(), 0..1);
      }
    }

    self.queue.submit(iter::once(encoder.finish()));

    output.present();

    Ok(())
  }

  fn create_text_buffer(&mut self, text_vertices: &[TextVertex]) {
    self.text_buffer = Some(self.device.create_buffer_init(&BufferInitDescriptor {
      label: Some("Text Vertices"),
      contents: text_vertices.as_bytes(),
      usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    }));
  }
}

struct ChunkGraphicsResources<'a> {
  device: &'a Device,
}

fn regenerate_chunk(renderer: &ChunkGraphicsResources<'_>, mesh: &mut ChunkMesh) -> (Buffer, u32) {
  let vertices = mesh.generate_vertices();

  let vertex_buffer = renderer.device.create_buffer_init(&BufferInitDescriptor {
    label: Some("Chunk Vertex Buffer"),
    contents: vertices.as_bytes(),
    usage: BufferUsages::VERTEX,
  });
  let vertices_len = vertices.len().coerce();

  (vertex_buffer, vertices_len)
}

struct ChunkRender {
  mesh: ChunkMesh,
  vertex_buffer: Buffer,
  vertices_len: u32,
}

impl ChunkRender {
  fn load(renderer: &ChunkGraphicsResources<'_>, chunk: &Chunk) -> Self {
    let mut mesh = ChunkMesh::generate(chunk);
    let (vertex_buffer, vertices_len) = regenerate_chunk(renderer, &mut mesh);

    Self {
      mesh,
      vertex_buffer,
      vertices_len,
    }
  }

  fn create_block(
    &mut self,
    renderer: &ChunkGraphicsResources<'_>,
    chunk: &Chunk,
    block_position: BlockPosition,
  ) {
    self.mesh.update_incremental(chunk, block_position);

    let (vertex_buffer, vertices_len) = regenerate_chunk(renderer, &mut self.mesh);

    self.vertex_buffer = vertex_buffer;
    self.vertices_len = vertices_len;
  }

  fn destroy_block(
    &mut self,
    renderer: &ChunkGraphicsResources<'_>,
    chunk: &Chunk,
    block_position: BlockPosition,
  ) {
    self.mesh.update_incremental(chunk, block_position);

    let (vertex_buffer, vertices_len) = regenerate_chunk(renderer, &mut self.mesh);

    self.vertex_buffer = vertex_buffer;
    self.vertices_len = vertices_len;
  }
}
