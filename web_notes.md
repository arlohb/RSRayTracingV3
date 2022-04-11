# WebGPU

[Implementation Status](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status)

This page also contains examples such as:

- https://austin-eng.com/webgpu-samples/samples/computeBoids

The page describes chrome canary, but I don't think this is available on linux. Instead, I got it working on chrome dev (on the AUR as google-chrome-dev).

I had to enable the flags #enable-vulkan and #enable-unsage-webgpu.

The web gpu flag is of course unsafe, but will be fine for testing.

I haven't decided whether I want to get this working in the web, as it would be a lot of work and I may run into a limitation I can't get past. Even then, it wouldn't work on any stable browsers right now, and it could be years until it is. In the meantime, the API will likely change and break my code.

It would be fun though...

Nonetheless, a major code change would be needed, as of course I can't have multiple windows in web.

I'll come back to this and check the status at a later date.
