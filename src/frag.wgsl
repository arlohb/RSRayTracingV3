struct Data {
  width: u32;
  height: u32;
};

[[group(0), binding(0)]]
var<storage, read> data: Data;

[[stage(fragment)]]
fn fs_main([[builtin(position)]] coord_in: vec4<f32>) -> [[location(0)]] vec4<f32> {
  return vec4<f32>(coord_in.x / f32(data.width), coord_in.y / f32(data.height), 0.0, 1.0);
}
