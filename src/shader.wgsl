struct VertexIn {
  @location(0) pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) color: vec4<f32>,
}

struct VertexOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) world: vec3<f32>,
  @location(2) normal: vec3<f32>,
}

@vertex fn vtx(in: VertexIn) -> VertexOut {
  return VertexOut(
    camera.mvp * vec4(in.pos, 1.0),
    in.color,
    in.pos,
    normalize(in.normal),
  );
}

@fragment fn frag(in: VertexOut) -> @location(0) vec4<f32> {
  var color = 0.05 * in.color.rgb;

  color += ambient(in.color.rgb, in.world, in.normal);
  color += lighting(in.color.rgb, in.world, in.normal);

  return vec4(color, in.color.a);
}
