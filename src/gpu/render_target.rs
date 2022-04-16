pub struct RenderTarget {
  pub render_view: wgpu::TextureView,
  pub id: Option<egui::TextureId>,
  pub size: (u32, u32),
}

impl RenderTarget {
  pub fn create_render_view(shared_gpu: &super::SharedGpu, size: (u32, u32)) -> wgpu::TextureView {
    let render_descriptor = wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      usage: wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::TEXTURE_BINDING,
      label: None,
    };

    let render_texture = shared_gpu.device.create_texture(&render_descriptor);
    let render_view = render_texture.create_view(&Default::default());

    render_view
  }

  pub fn new(shared_gpu: &super::SharedGpu, initial_size: (u32, u32)) -> RenderTarget {
    RenderTarget {
      id: None,
      size: initial_size,
      render_view: RenderTarget::create_render_view(shared_gpu, initial_size),
    }
  }

  pub fn update(
    &mut self,
    device: &wgpu::Device,
    egui_rpass: &mut egui_wgpu_backend::RenderPass,
  ) {
    match self.id {
      Some(id) => {
        egui_rpass.update_egui_texture_from_wgpu_texture(
          device,
          &self.render_view,
          wgpu::FilterMode::Nearest,
          id,
        ).unwrap();
      }
      None => {
        self.id = Some(egui_rpass.egui_texture_from_wgpu_texture(
          device,
          &self.render_view,
          wgpu::FilterMode::Nearest,
        ));
      }
    }
  }

  pub fn resize(
    &mut self,
    shared_gpu: &super::SharedGpu,
    size: (u32, u32),
  ) {
    self.size = size;
    self.render_view = RenderTarget::create_render_view(shared_gpu, size);
  }
}
