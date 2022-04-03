struct Data {
    numbers: [[stride(4)]] array<f32>;
};

[[group(0), binding(0)]]
var<storage, read_write> data: Data;

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    // TODO: a more interesting computation than this.
    data.numbers[global_id.x] = data.numbers[global_id.x] + 42.0;
}