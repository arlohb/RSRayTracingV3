struct Data {
    test: [[stride(4)]] array<f32>;
};

[[group(0), binding(0)]]
var<storage, read_write> data: Data;

[[stage(compute), workgroup_size(16, 16, 1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    data.test[global_id.x] = data.test[global_id.x] + 42.0;
}