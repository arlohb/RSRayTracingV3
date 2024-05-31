struct Material { // 48
  float3 colour; // 12
  int _0;// 4
  float3 emission; // 12
  float emission_strength; // 4
  float metallic; // 4
  float roughness; // 4
  int _1[2]; // 8
};

struct Geometry { // 56
  uint option; // 4
  int _0[3]; // 12
  float3 center; // 16
  float3 vec_data; // 12
  float f32_data; // 4
  uint data[2]; // 8
};

struct Object { // 112
  Material material; // 48
  Geometry geometry; // 64
};

struct Light {
  uint options;
  int _0[3];
  float3 colour;
  int _1;
  float3 vec_data;
  int _2;
};

struct Config {
  float3 position;
  int _0;
  float3 forward;
  int _1;
  float3 right;
  int _2;
  float3 up;
  int _3;
  float3 background_colour;
  int _4;
  float3 ambient_light;
  float fov;
  uint reflection_limit;
  uint width;
  uint height;
};

struct FrameData {
  float2 jitter;
  uint progressive_count;
};

// difference between StructuredBuffer and ConstantBuffer
// https://www.gamedev.net/forums/topic/624529-structured-buffers-vs-constant-buffers/4937832/
// https://docs.microsoft.com/en-us/windows/win32/direct3d12/resource-binding-in-hlsl#constant-buffers

// I don't think I need explicit register bindings,
// but I'll leave it here for now.

// register(b0, space0) = binding 0, group 0

StructuredBuffer<Object> objects : register(b0);
StructuredBuffer<Light> lights : register(b1);
ConstantBuffer<Config> config : register(b2);
SamplerState s_tex : register(b3);
Texture2D<float4> t_hdri : register(b4);
Texture2D<float4> t_render : register(b5);
Texture2D<float4> t_random : register(b6);
ConstantBuffer<FrameData> frame_data : register(b7);

static uint object_count;

void inputs_init() {
  uint stride;
  objects.GetDimensions(object_count, stride);
}
