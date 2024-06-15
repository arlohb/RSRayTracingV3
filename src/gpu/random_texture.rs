use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

use image::EncodableLayout;

pub struct RandomTexture {
    width: u32,
    height: u32,
    data: RwLock<Vec<f32>>,
    needs_write: AtomicBool,

    texture: wgpu::Texture,
    size: wgpu::Extent3d,

    pub view: wgpu::TextureView,
}

impl RandomTexture {
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Arc<Self> {
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

        let this = Self {
            width,
            height,
            data: RwLock::new(Vec::new()),
            needs_write: AtomicBool::new(true),

            texture,
            size,

            view,
        };

        *this.data.write().expect("RwLock poisoned") = this.generate_data();

        let this_ref = Arc::new(this);

        let this_clone = this_ref.clone();
        thread::spawn(move || {
            let this = &this_clone;
            loop {
                thread::sleep(Duration::from_millis(4));

                *this.data.write().expect("RwLock poisoned") = this.generate_data();
                this.needs_write.store(true, Ordering::Relaxed);
            }
        });

        this_ref
    }

    #[must_use]
    pub fn generate_data(&self) -> Vec<f32> {
        let length = (self.width * self.height) as usize;
        let mut data = vec![0f32; length];
        let mut rng = fastrand::Rng::new();

        data.iter_mut().for_each(|v| *v = rng.f32());

        data
    }

    pub fn write(&self, queue: &wgpu::Queue) {
        puffin::profile_function!();

        let wgpu::Extent3d { width, height, .. } = self.size;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // Get the raw bytes to the float vector
            self.data.read().expect("RwLock poisoned").as_bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            self.size,
        );
    }

    pub fn maybe_write(&self, queue: &wgpu::Queue) {
        if self.needs_write.load(Ordering::Relaxed) {
            self.write(queue);
            self.needs_write.store(false, Ordering::Relaxed);
        }
    }
}
