// use inline_spirv::include_spirv;
use std::borrow::Cow;

pub fn vert_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
  // let spirv = include_spirv!("src/gpu/vert.hlsl", vert, hlsl, entry="vs_main");

  device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    label: None,
    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
      include_str!("vert.wgsl"),
    )),
    // source: wgpu::ShaderSource::SpirV(Cow::Borrowed(
    //   // spirv.as_binary(),
    //   spirv
    // )),
  })
}

fn parse_shader(bounce_limit: u32) -> String {
  let mut dict = std::collections::HashMap::<&str, String>::new();
  dict.insert("bounce_limit", format!("{}u", bounce_limit));
  dict.insert("metallic_stack_values", "0., ".repeat(bounce_limit as usize));
  dict.insert("reflection_colour_stack_values", "vec3<f32>(0., 0., 0.), ".repeat(bounce_limit as usize));

  let frag = include_str!("frag.wgsl").to_string();

  let frag_parsed = frag.lines()
    .filter(|line| !line.contains("////"))
    .map(|line| {
      let split = line.split("//").collect::<Vec<_>>();
      if split.len() == 3 {
        match dict.get(split[1].trim()) {
          Some(template_value) => split[0].to_string() + template_value + split[2],
          None => line.to_string(),
        }
      } else {
        line.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join("\n");
  
  frag_parsed
}

pub fn frag_shader(device: &wgpu::Device, bounce_limit: u32) -> wgpu::ShaderModule {
  device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    label: None,
    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
      parse_shader(bounce_limit).as_str(),
    )),
  })
}
