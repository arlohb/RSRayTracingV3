struct Data {
    texture: [[stride(4)]] array<f32>;
};

[[group(0), binding(0)]]
var<storage, read_write> data: Data;

[[group(0), binding(1)]]
var t_render: texture_storage_2d<rgba8uint, read_write>;

[[stage(compute), workgroup_size(16, 16, 1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    data.texture[global_id.x] = data.texture[global_id.x] + 42.0;


    let point = vec2<i32>(0, 0);
    let colour = vec4<u32>(0u, 0u, 0u, 0u);

    textureStore(t_render, point, colour);
}