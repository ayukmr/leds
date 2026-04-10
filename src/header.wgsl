struct Camera {
  mvp: mat4x4<f32>,
  pos: vec3<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;

struct Light {
  pos: vec3<f32>,
  yaw: f32,
  color: vec3<f32>,
}
struct Lights {
  lights: array<Light, 50>,
  len: u32,
}
@group(1) @binding(0) var<uniform> lights: Lights;

fn rotate(v: vec3<f32>, yaw: f32) -> vec3<f32> {
  let s = sin(yaw);
  let c = cos(yaw);

  return vec3<f32>(c * v.x + s * v.z, v.y, -s * v.x + c * v.z);
}

fn ambient(color: vec3<f32>, world: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
  let view_dir = normalize(camera.pos - world);

  let light_dir = normalize(vec3(4.0, 2.0, 1.0));
  let half_dir = normalize(view_dir + light_dir);

  let diffuse = max(dot(normal, light_dir), 0.0);
  let specular = pow(max(dot(normal, half_dir), 0.0), 128.0);

  return 0.4 * (diffuse + specular) * vec3(0.6, 0.6, 0.65) * color;
}

fn lighting(color: vec3<f32>, world: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
  let view_dir = normalize(camera.pos - world);
  var c = vec3(0.0);

  for (var i = 0u; i < lights.len; i++) {
    let light = lights.lights[i];

    let to_light = light.pos - world;
    let dist = length(to_light);

    let light_dir = to_light / dist;
    let half_dir = normalize(view_dir + light_dir);

    let attenuation = 1.0 / (1.0 + 0.3 * dist + 2500.0 * dist * dist);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 128.0);

    c += 10.0 * attenuation * (diffuse_strength + specular_strength) * light.color * color;
  }

  return c;
}
