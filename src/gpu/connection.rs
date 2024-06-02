use image::{EncodableLayout, GenericImageView};
use nalgebra::Vector2;

use crate::{
    ray_tracer::{Scene, Vec3},
    utils::bytes::{bytes_concat, bytes_concat_owned, AsBytes},
};

pub struct FrameData {
    pub jitter: Vector2<f32>,
    pub progressive_count: u32,
}

impl FrameData {
    const BUFFER_SIZE: usize = 16;
    const JITTER_STRENGTH: f32 = 0.99;

    pub const fn new(jitter: Vector2<f32>) -> Self {
        Self {
            jitter,
            progressive_count: 0,
        }
    }
}

impl AsBytes<{ Self::BUFFER_SIZE }> for FrameData {
    fn as_bytes(&self) -> [u8; Self::BUFFER_SIZE] {
        bytes_concat(
            [
                &self.jitter.as_bytes(),
                self.progressive_count.to_le_bytes().as_slice(),
            ]
            .into_iter(),
        )
    }
}

pub struct Connection {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    pub last_scene: Scene,
    pub last_size: (u32, u32),
    pub frame_data: FrameData,

    pub objects: wgpu::Buffer,
    pub lights: wgpu::Buffer,
    pub config: wgpu::Buffer,
    pub frame_data_buffer: wgpu::Buffer,

    pub random_texture: wgpu::Texture,
    pub random_texture_size: wgpu::Extent3d,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Connection {
    pub const VERTICES_NUM: usize = 4;
    pub const VERTICES: [Vec3; Self::VERTICES_NUM] = [
        Vec3::new(-1., -1., 0.),
        Vec3::new(-1., 1., 0.),
        Vec3::new(1., 1., 0.),
        Vec3::new(1., -1., 0.),
    ];
    pub const INDICES_NUM: usize = 6;
    pub const INDICES: [u16; Self::INDICES_NUM] = [0, 2, 1, 0, 3, 2];

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
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    fn load_image(file_name: &str) -> ((u32, u32), Vec<u8>) {
        let reader = image::io::Reader::open(file_name)
            .unwrap_or_else(|_| panic!("Can't find image: {file_name}"));
        let im = reader
            .decode()
            .unwrap_or_else(|_| panic!("Image invalid format: {file_name}"));

        (im.dimensions(), im.into_rgba32f().as_bytes().to_vec())
    }

    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    fn load_hdri(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
        let hdri_file = "./assets/table_mountain_1_8k.exr";

        let (size, hdri_bytes) = Self::load_image(hdri_file);

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
            view_formats: &[wgpu::TextureFormat::Rgba32Float],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            hdri_bytes.as_slice(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(16 * size.0),
                rows_per_image: Some(size.1),
            },
            texture_size,
        );

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_random_texture(
        device: &wgpu::Device,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Extent3d) {
        let texture_size = wgpu::Extent3d {
            width: 600,
            height: 600,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("random_texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            view_formats: &[wgpu::TextureFormat::R32Float],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, texture_view, texture_size)
    }

    fn write_random_texture(&self, queue: &wgpu::Queue) {
        puffin::profile_function!();

        let wgpu::Extent3d { width, height, .. } = self.random_texture_size;
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
                    texture: &self.random_texture,
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
                self.random_texture_size,
            );
        }
    }

    fn create_buffers(device: &wgpu::Device) -> [wgpu::Buffer; 4] {
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
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let frame_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: FrameData::BUFFER_SIZE as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        [objects, lights, config, frame_data_buffer]
    }

    #[must_use]
    pub const fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: 16,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }

    fn create_model_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        use wgpu::util::DeviceExt;

        let vertex_bytes = Self::VERTICES
            .iter()
            .flat_map(|v| bytes_concat::<16>([&v.as_bytes(), [0u8; 4].as_slice()].into_iter()))
            .collect::<Vec<_>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: &vertex_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_bytes = bytes_concat_owned::<2, { Self::INDICES_NUM * 2 }>(
            Self::INDICES.iter().map(|index| index.to_le_bytes()),
        );

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: &index_bytes,
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }

    #[must_use]
    pub fn new(
        scene: &Scene,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_view: &wgpu::TextureView,
    ) -> Self {
        let sampler = Self::create_sampler(device);
        let hdri_texture_view = Self::load_hdri(device, queue);
        let (random_texture, random_texture_view, random_texture_size) =
            Self::create_random_texture(device);

        let [objects, lights, config, frame_data_buffer] = Self::create_buffers(device);

        let bind_group_layout = Self::bind_group_layout(device);

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
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&hdri_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(render_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&random_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: frame_data_buffer.as_entire_binding(),
                },
            ],
        });

        let (vertex_buffer, index_buffer) = Self::create_model_buffers(device);

        Self {
            bind_group,
            bind_group_layout,

            last_scene: scene.clone(),
            last_size: (0, 0),
            frame_data: FrameData::new(Vector2::zeros()),

            objects,
            lights,
            config,
            frame_data_buffer,

            random_texture,
            random_texture_size,

            vertex_buffer,
            index_buffer,
        }
    }

    pub fn update_buffers(&mut self, queue: &wgpu::Queue, size: (u32, u32), scene: &Scene) {
        puffin::profile_function!();

        if (scene != &self.last_scene) | (size != self.last_size) {
            puffin::profile_scope!("serialize_scene");

            let (object_bytes, light_bytes, config_bytes) = scene.as_bytes(size.0, size.1);

            queue.write_buffer(&self.objects, 0, object_bytes.as_slice());
            queue.write_buffer(&self.lights, 0, light_bytes.as_slice());
            queue.write_buffer(&self.config, 0, config_bytes.as_slice());

            self.last_scene = scene.clone();
            self.last_size = size;

            self.frame_data.progressive_count = 0;
        } else {
            self.frame_data.progressive_count += 1;
        }

        self.write_random_texture(queue);

        {
            puffin::profile_scope!("jitter_gen");
            self.frame_data.jitter.x = rand::random::<f32>().mul_add(
                FrameData::JITTER_STRENGTH,
                -(FrameData::JITTER_STRENGTH / 2.),
            );
            self.frame_data.jitter.y = rand::random::<f32>().mul_add(
                FrameData::JITTER_STRENGTH,
                -(FrameData::JITTER_STRENGTH / 2.),
            );
        }

        {
            puffin::profile_scope!("serialize_frame_data");
            queue.write_buffer(
                &self.frame_data_buffer,
                0,
                self.frame_data.as_bytes().as_slice(),
            );
        }
    }
}
