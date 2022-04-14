use std::borrow::Cow;

pub fn vert_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
  device.create_shader_module(&wgpu::ShaderModuleDescriptor {
    label: None,
    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
      include_str!("vert.wgsl"),
    )),
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
        let template_name = split[1].trim();
        let template_value = dict[template_name].as_str();
        split[0].to_string() + template_value + split[2]
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
