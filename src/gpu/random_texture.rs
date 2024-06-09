use image::EncodableLayout;

pub struct RandomTexture {
    texture: wgpu::Texture,
    size: wgpu::Extent3d,

    pub view: wgpu::TextureView,
}

impl RandomTexture {
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let size = wgpu::Extent3d {
            width: 600,
            height: 600,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("random_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            view_formats: &[wgpu::TextureFormat::R32Float],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            size,
            view,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue) {
        puffin::profile_function!();

        let wgpu::Extent3d { width, height, .. } = self.size;
        let length = (width * height) as usize;
        let mut data = vec![0f32; length];

        let mut rng = fastrand::Rng::new();

        {
            puffin::profile_scope!("random_gen");
            data.iter_mut().for_each(|v| *v = rng.f32());
        }

        {
            puffin::profile_scope!("random_write");
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                // Get the raw bytes to the float vector
                data.as_bytes(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                self.size,
            );
        }
    }
}
