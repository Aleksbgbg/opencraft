const OVERDRAW_FACTOR: f32 = 1.001;
const LINE_WIDTH_FRACTION: f32 = 0.025;
const LINE_COLOUR: f32 = 0.025;

@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

struct VertexInput {
  @location(0) position: vec3<f32>,
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) vertex_position: vec3<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.position = transform * vec4<f32>(vertex.position * vec3(OVERDRAW_FACTOR), 1.0);
  out.vertex_position = vertex.position;
  return out;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  // If we are at a cube edge, the sum of the components of at_edge >= 2
  let vert_norm = abs(vertex.vertex_position) * 2.0;
  let at_edge = step(vec3(1.0 - LINE_WIDTH_FRACTION), vert_norm);
  if ((at_edge.x + at_edge.y + at_edge.z) < 2.0) {
    discard;
  }

  return vec4(vec3(LINE_COLOUR), 1.0);
}
