#[derive(Clone, Copy)]
pub struct RenderTexture {
  pub id: Option<egui::TextureId>,
  pub size: (u32, u32),
}

impl RenderTexture {
  pub fn new(initial_size: (u32, u32)) -> RenderTexture {
    RenderTexture {
      id: None,
      size: initial_size,
    }
  }

  pub fn update(
    &mut self,
    device: &wgpu::Device,
    egui_rpass: &mut egui_wgpu_backend::RenderPass,
    input: &wgpu::TextureView,
  ) {
    match self.id {
      Some(id) => {
        egui_rpass.update_egui_texture_from_wgpu_texture(
          device,
          input,
          wgpu::FilterMode::Nearest,
          id,
        ).unwrap();
      }
      None => {
        self.id = Some(egui_rpass.egui_texture_from_wgpu_texture(
          device,
          input,
          wgpu::FilterMode::Nearest,
        ));
      }
    }
  }
}
