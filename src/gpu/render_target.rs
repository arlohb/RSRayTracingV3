pub struct RenderTarget {
    pub render_texture: wgpu::Texture,
    pub render_view: wgpu::TextureView,
    pub id: Option<egui::TextureId>,
    pub size: (u32, u32),
}

impl RenderTarget {
    pub fn create_render_texture(
        device: &wgpu::Device,
        size: (u32, u32),
    ) -> (wgpu::Texture, wgpu::TextureView) {
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
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            label: None,
        };

        let render_texture = device.create_texture(&render_descriptor);
        let render_view = render_texture.create_view(&Default::default());

        (render_texture, render_view)
    }

    pub fn new(device: &wgpu::Device, initial_size: (u32, u32)) -> RenderTarget {
        let (render_texture, render_view) =
            RenderTarget::create_render_texture(device, initial_size);

        RenderTarget {
            id: None,
            size: initial_size,
            render_texture,
            render_view,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        egui_renderer: &mut egui_wgpu::renderer::Renderer,
    ) {
        match self.id {
            Some(id) => {
                egui_renderer.update_egui_texture_from_wgpu_texture(
                    device,
                    &self.render_view,
                    wgpu::FilterMode::Nearest,
                    id,
                );
            }
            None => {
                self.id = Some(egui_renderer.register_native_texture(
                    device,
                    &self.render_view,
                    wgpu::FilterMode::Nearest,
                ));
            }
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        self.size = size;
        (self.render_texture, self.render_view) = RenderTarget::create_render_texture(device, size);
    }
}
