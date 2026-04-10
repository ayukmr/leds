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

@vertex fn vtx(@builtin(instance_index) idx: u32, in: VertexIn) -> VertexOut {
  let light = lights.lights[idx];
  let pos = light.pos + rotate(in.pos, light.yaw);
  let wnormal = normalize(rotate(in.normal, light.yaw));

  return VertexOut(
    camera.mvp * vec4(pos, 1.0),
    vec4(light.color, 1.0),
    pos,
    wnormal,
  );
}

@fragment fn frag(in: VertexOut) -> @location(0) vec4<f32> {
  var color = 0.8 * in.color.rgb;

  color += lighting(in.color.rgb, in.world, in.normal);

  return vec4(color, in.color.a);
}
