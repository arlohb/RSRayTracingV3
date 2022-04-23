# WebGPU

[Implementation Status](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status)

This page also contains examples such as:

- https://austin-eng.com/webgpu-samples/samples/computeBoids

The page describes chrome canary, but I don't think this is available on linux. Instead, I got it working on chrome dev (on the AUR as google-chrome-dev).

I had to enable the flags #enable-vulkan and #enable-unsage-webgpu.

The web gpu flag is of course unsafe, but will be fine for testing.

# WebGL2

Wgpu has experimental support web WebGL2. This is the way forward for now.

Unfortunately, this prevents me from switching to a compute shader as WebGL2 doesn't support compute shaders and development stopped in favour of WebGPU.
