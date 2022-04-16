#[derive(Clone, Copy)]
pub struct RenderTexture {
  pub id: Option<egui::TextureId>,
  pub width: u32,
  pub height: u32,
}

impl RenderTexture {
  pub fn new() -> RenderTexture {
    RenderTexture {
      id: None,
      width: 500,
      height: 500,
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

impl Default for RenderTexture {
  fn default() -> RenderTexture {
    RenderTexture::new()
  }
}
