struct VertexInput {
  @location(0) screen_position: vec2<f32>,
  @location(1) font_atlas_position: vec2<f32>,
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) texture_coordinate: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.position = vec4<f32>(vertex.screen_position, 0.0, 1.0);
  out.texture_coordinate = vertex.font_atlas_position;
  return out;
}

@group(0) @binding(0)
var font_atlas: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  return vec4(vec3(1.0), textureSample(font_atlas, texture_sampler, vertex.texture_coordinate).r);
}
