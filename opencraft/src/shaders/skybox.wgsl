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

const BLACK: vec3<f32> = vec3(0.0);
const PLAINS_SKY_BLUE: vec3<f32> = vec3(0.471, 0.655, 1.0); // #78A7FF

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  let weight = (normalize(vertex.world_position).y + 1.0) / 2.0;
  return vec4(mix(BLACK, PLAINS_SKY_BLUE, weight), 1.0);
}
