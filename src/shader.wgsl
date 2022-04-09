[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
  let index = i32(vertex_index);
  // var x = 0.;
  var y = 0.;

  // 1, -1, 1, -1, 1, -1
  let x = f32((index % 2) * -1 + 1 - (index % 2));

  if (index == 0) {
    y = -1.;
  } else if (index == 1) {
    y = -1.;
  } else if (index == 2) {
    y = 1.;
  } else if (index == 3) {
    y = -1.;
  } else if (index == 4) {
    y = 1.;
  } else if (index == 5) {
    y = 1.;
  }

  return vec4<f32>(x, y, 0., 1.);
}

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
