struct Material {
  colour: vec3<f32>;
  specular: f32;
  metallic: f32;
};

struct Geometry {
  option: u32;
  center: vec3<f32>;
  vec_data: vec3<f32>;
  data: array<u32, 3>;
};

struct Object {
  material: Material;
  geometry: Geometry;
};

struct Objects {
  objects: array<Object>;
};

struct Light {
  options: u32;
  colour: vec3<f32>;
  vec_data: vec3<f32>;
};

struct Lights {
  lights: array<Light>;
};

struct Camera { // 40
  position: vec3<f32>; // 16
  direction: vec3<f32>; // 16
  fov: f32; // 4
  // implicit padding // 4
};

struct Config {
  width: u32; // 4
  height: u32; // 4
  camera: Camera; // 40
  background_colour: vec3<f32>; // 16
  ambient_light: vec3<f32>; // 16
};

[[group(0), binding(0)]]
var<storage, read> objects: Objects;

[[group(0), binding(1)]]
var<storage, read> lights: Lights;

[[group(0), binding(2)]]
var<storage, read> config: Config;

[[stage(fragment)]]
fn fs_main([[builtin(position)]] coord_in: vec4<f32>) -> [[location(0)]] vec4<f32> {
  return vec4<f32>(coord_in.x / f32(config.width), coord_in.y / f32(config.height), 0.0, 1.0);
}
