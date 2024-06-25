@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

struct VertexInput {
  @location(0) position: vec3<f32>,
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) world_position: vec3<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.position = transform * vec4<f32>(vertex.position, 1.0);
  out.world_position = vertex.position;
  return out;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  if (vertex.world_position.y < 0.0) {
    discard;
  }

  // Plains sky colour: #78A7FF
  return vec4(0.471, 0.655, 1.0, 1.0);
}
