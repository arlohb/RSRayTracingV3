use std::{sync::Arc, thread, time::Duration};

use image::EncodableLayout;

/// Stores a random texture,
/// which is re-generated and written to the GPU on a separate thread.
pub struct RandomTexture {
    width: u32,
    height: u32,
    data: Vec<f32>,

    queue: Arc<wgpu::Queue>,
    texture: wgpu::Texture,
    size: wgpu::Extent3d,
}

impl RandomTexture {
    /// Start the thread and return a view to the random texture.
    #[must_use]
    pub fn start(device: &wgpu::Device, queue: Arc<wgpu::Queue>) -> wgpu::TextureView {
        let width = 600;
        let height = 600;

        let size = wgpu::Extent3d {
            width,
            height,
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

        let mut this = Self {
            width,
            height,
            data: Vec::new(),

            queue,
            texture,
            size,
        };

        this.generate_data();
        this.write();

        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(4));

            this.generate_data();
            this.write();
        });

        view
    }

    /// Generate the random data.
    fn generate_data(&mut self) {
        let length = (self.width * self.height) as usize;
        let mut data = vec![0f32; length];
        let mut rng = fastrand::Rng::new();

        data.iter_mut().for_each(|v| *v = rng.f32());

        self.data = data;
    }

    /// Write the texture to the GPU
    fn write(&self) {
        puffin::profile_function!();

        let wgpu::Extent3d { width, height, .. } = self.size;

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // Get the raw bytes to the float vector
            self.data.as_bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            self.size,
        );

        self.queue.submit(std::iter::empty());
    }
}
