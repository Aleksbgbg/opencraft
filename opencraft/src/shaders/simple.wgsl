@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) texture_coordinate: vec2<f32>,
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) texture_coordinate: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.position = transform * vec4<f32>(vertex.position, 1.0);
  out.texture_coordinate = vertex.texture_coordinate;
  return out;
}

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  return textureSample(texture, texture_sampler, vertex.texture_coordinate);
}
