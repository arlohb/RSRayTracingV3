use inline_spirv::include_spirv;
use std::borrow::Cow;

/// Load the vertex shader.
#[must_use]
pub fn vert_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    let spirv = include_spirv!(
        "src/shaders/vert.hlsl",
        vert,
        hlsl,
        // This issue is only caused when debug is on
        // https://github.com/gfx-rs/wgpu/issues/4532
        no_debug,
        entry = "vs_main"
    );

    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::SpirV(Cow::Borrowed(spirv)),
    })
}

/// Load the fragment shader.
#[must_use]
pub fn frag_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    let spirv = include_spirv!(
        "src/shaders/frag.hlsl",
        frag,
        hlsl,
        // This issue is only caused when debug is on
        // https://github.com/gfx-rs/wgpu/issues/4532
        no_debug,
        entry = "fs_main"
    );

    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::SpirV(Cow::Borrowed(spirv)),
    })
}
