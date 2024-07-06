struct VertexInput {
  @builtin(vertex_index) index: u32,
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) texture_coordinate: vec2<f32>,
}

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
  var vertices = array(
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0),
  );

  let screen_position = vertices[vertex.index];

  var out: VertexOutput;
  out.position = vec4<f32>(screen_position, 0.0, 1.0);
  out.texture_coordinate = (vec2(screen_position.x, -screen_position.y) + 1.0) / 2.0;
  return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  return textureSample(texture, texture_sampler, vertex.texture_coordinate);
}
