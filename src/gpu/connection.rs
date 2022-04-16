use std::sync::{Arc, Mutex};
use image::{EncodableLayout, GenericImageView};

use crate::ray_tracer::Scene;

pub struct Connection {
  pub bind_group_layout: wgpu::BindGroupLayout,
  pub bind_group: wgpu::BindGroup,

  pub scene: Arc<Mutex<Scene>>,
  pub last_scene: Scene,
  pub last_size: (u32, u32),

  pub objects: wgpu::Buffer,
  pub lights: wgpu::Buffer,
  pub config: wgpu::Buffer,
}

impl Connection {
  fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: None,
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 3,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 4,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
          count: None,
        },
      ],
    })
  }

  fn load_hdri(device: &wgpu::Device, queue: &wgpu::Queue) -> (wgpu::TextureView, wgpu::Sampler) {
    let reader = image::io::Reader::open("./assets/table_mountain_1_8k.exr").unwrap();
    let hdri = reader.decode().unwrap();
    let size = hdri.dimensions();

    let texture_size = wgpu::Extent3d {
      width: size.0,
      height: size.1,
      depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("hdri_texture"),
      size: texture_size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba32Float,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });

    queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      hdri.to_rgba32f().as_bytes(),
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: std::num::NonZeroU32::new(16 * size.0),
        rows_per_image: std::num::NonZeroU32::new(size.1),
      },
      texture_size,
    );

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Nearest,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });

    (texture_view, sampler)
  }

  fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
    let objects = device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: Scene::BUFFER_SIZE.0 as u64,
      mapped_at_creation: false,
      usage: wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC,
    });

    let lights = device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: Scene::BUFFER_SIZE.1 as u64,
      mapped_at_creation: false,
      usage: wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC,
    });

    let config = device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: Scene::BUFFER_SIZE.2 as u64,
      mapped_at_creation: false,
      usage: wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC,
    });

    (objects, lights, config)
  }

  pub fn new(scene: Arc<Mutex<Scene>>, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let (hdri_texture_view, hdri_sampler) = Connection::load_hdri(device, queue);

    let (objects, lights, config) = Connection::create_buffers(device);

    let bind_group_layout = Connection::bind_group_layout(device);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: objects.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: lights.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: config.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 3,
          resource: wgpu::BindingResource::TextureView(&hdri_texture_view),
        },
        wgpu::BindGroupEntry {
          binding: 4,
          resource: wgpu::BindingResource::Sampler(&hdri_sampler),
        },
      ],
    });

    Connection {
      bind_group,
      bind_group_layout,
      scene: scene.clone(),
      last_scene: scene.lock().unwrap().clone(),
      last_size: (0, 0),
      objects,
      lights,
      config,
    }
  }

  pub fn update_buffer(&mut self, queue: &wgpu::Queue, size: (u32, u32)) {
    let scene = self.scene.lock().unwrap().clone();

    if (scene != self.last_scene) | (size != self.last_size) {
      let (object_bytes, light_bytes, config_bytes) =
        scene.as_bytes(size.0, size.1);

      queue.write_buffer(&self.objects, 0, object_bytes.as_slice());
      queue.write_buffer(&self.lights, 0, light_bytes.as_slice());
      queue.write_buffer(&self.config, 0, config_bytes.as_slice());
    }

    self.last_scene = scene;
    self.last_size = size;
  }
}
