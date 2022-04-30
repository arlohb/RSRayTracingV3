float4 vs_main(float3 input : POSITION) : SV_POSITION {
  return float4(input, 1.0);
}
