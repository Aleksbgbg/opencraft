struct Quad {
  left: f32,
  right: f32,
  top: f32,
  bot: f32,
}

@group(1) @binding(0)
var<uniform> crosshair_quad: Quad;

struct VertexInput {
  @builtin(vertex_index) index: u32,
}

struct Vertex {
  screen_position: vec2<f32>,
  texture_coordinate: vec2<f32>,
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) texture_coordinate: vec2<f32>,
  @location(1) framebuffer_texture_coordinate: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
  var vertices = array(
    Vertex(
      vec2(crosshair_quad.left, crosshair_quad.top),
      vec2(0.0, 1.0),
    ),
    Vertex(
      vec2(crosshair_quad.right, crosshair_quad.top),
      vec2(1.0, 1.0),
    ),
    Vertex(
      vec2(crosshair_quad.left, crosshair_quad.bot),
      vec2(0.0, 0.0),
    ),
    Vertex(
      vec2(crosshair_quad.right, crosshair_quad.bot),
      vec2(1.0, 0.0),
    ),
  );

  let vertex = vertices[input.index];

  var out: VertexOutput;
  out.position = vec4<f32>(vertex.screen_position, 0.0, 1.0);
  out.texture_coordinate = vertex.texture_coordinate;
  out.framebuffer_texture_coordinate =
    (vec2(vertex.screen_position.x, -vertex.screen_position.y) + 1.0) / 2.0;
  return out;
}

@group(0) @binding(0)
var framebuffer: texture_2d<f32>;
@group(0) @binding(1)
var framebuffer_sampler: sampler;

@group(1) @binding(1)
var crosshair_alpha: texture_2d<f32>;
@group(1) @binding(2)
var crosshair_alpha_sampler: sampler;

const CROSSHAIR_COLOR = vec3(1.0, 1.0, 1.0);

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  let framebuffer = textureSample(
    framebuffer,
    framebuffer_sampler,
    vertex.framebuffer_texture_coordinate
  ).rgb;
  let alpha = textureSample(
    crosshair_alpha,
    crosshair_alpha_sampler,
    vertex.texture_coordinate
  ).r;

  if is_similar_colour(CROSSHAIR_COLOR.r, framebuffer.r) ||
    is_similar_colour(CROSSHAIR_COLOR.b, framebuffer.g) ||
    is_similar_colour(CROSSHAIR_COLOR.g, framebuffer.b) {
    return vec4(CROSSHAIR_COLOR - framebuffer, alpha);
  } else {
    return vec4(CROSSHAIR_COLOR, alpha);
  }
}

const SIMILARITY_TOLERANCE = 0.05;

fn is_similar_colour(a: f32, b: f32) -> bool {
  return abs(a - b) < SIMILARITY_TOLERANCE;
}
