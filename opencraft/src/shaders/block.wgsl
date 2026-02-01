@group(0) @binding(0)
var<uniform> world_to_screen: mat4x4<f32>;

struct SelectedIndex {
  @size(16) index: u32,
}

@group(1) @binding(0)
var<uniform> selected_block: SelectedIndex;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) texture_coordinate: vec2<f32>,
  @location(2) line_coordinates: vec2<f32>,
  @location(3) block_index: u32,
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) texture_coordinate: vec2<f32>,
  @location(1) line_coordinates: vec2<f32>,
  @location(2) @interpolate(flat) alpha_multiplier: f32,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.position = world_to_screen * vec4<f32>(vertex.position, 1.0);
  out.texture_coordinate = vertex.texture_coordinate;
  out.line_coordinates = vertex.line_coordinates;
  out.alpha_multiplier = f32(selected_block.index == vertex.block_index);
  return out;
}

@group(0) @binding(1)
var texture: texture_2d<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;
@group(0) @binding(3)
var line_texture: texture_2d<f32>;
@group(0) @binding(4)
var line_texture_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  const OUTLINE_COLOUR: vec4<f32> = vec4(vec3(0.025), 1.0);

  let colour = textureSampleBias(texture, texture_sampler, vertex.texture_coordinate, 0.0);

  let line_alpha_1 = textureSample(
    line_texture,
    line_texture_sampler,
    vec2(vertex.line_coordinates.x, 0.5),
  ).r;
  let line_alpha_2 = textureSample(
    line_texture,
    line_texture_sampler,
    vec2(vertex.line_coordinates.y, 0.5),
  ).r;

  return mix(
    colour,
    OUTLINE_COLOUR,
    max(line_alpha_1, line_alpha_2) * vertex.alpha_multiplier,
  );
}
