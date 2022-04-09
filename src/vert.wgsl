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
