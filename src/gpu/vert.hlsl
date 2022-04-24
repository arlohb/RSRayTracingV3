#pragma vs vs_main;

struct VertexInput {
  float3 position: POSITION;
};

struct VertexOutput {
  float3 position: POSITION;
};

VertexOutput vs_main(VertexInput input) {
  VertexOutput output;
  output.position = input.position;
  return output;
}
