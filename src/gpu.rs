use std::{
  borrow::Cow,
  sync::{Arc, Mutex},
};
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::Window,
  platform::unix::EventLoopExtUnix,
};
use image::{EncodableLayout, GenericImageView};

use crate::ray_tracer::{Scene, SceneBuffers};

struct Gpu {
  window: Window,
  scene: Arc<Mutex<Scene>>,
  last_scene: Scene,
  last_size: winit::dpi::PhysicalSize<u32>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  surface: wgpu::Surface,
  render_pipeline: wgpu::RenderPipeline,
  bind_group: wgpu::BindGroup,
  scene_buffers: SceneBuffers,
  config: wgpu::SurfaceConfiguration,
}

impl Gpu {
  fn parse_shader(bounce_limit: u32) -> String {
    let mut dict = std::collections::HashMap::<&str, String>::new();
    dict.insert("bounce_limit", format!("{}u", bounce_limit));
    dict.insert("metallic_stack_values", "0., ".repeat(bounce_limit as usize));
    dict.insert("reflection_colour_stack_values", "vec3<f32>(0., 0., 0.), ".repeat(bounce_limit as usize));

    let vert = include_str!("vert.wgsl").to_string();

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
    
    vert + frag_parsed.as_str()
  }

  fn read_hdri() -> (image::DynamicImage, (u32, u32)) {
    let reader = image::io::Reader::open("./assets/table_mountain_1_8k.exr").unwrap();
    let hdri = reader.decode().unwrap();
    let size = hdri.dimensions();

    (hdri, size)
  }

  pub async fn new(
    window: Window,
    scene: Arc<Mutex<Scene>>,
  ) -> Gpu {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        // Request an adapter which can render to our surface
        compatible_surface: Some(&surface),
      })
      .await
      .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          features: wgpu::Features::BUFFER_BINDING_ARRAY
            | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY,
          limits: wgpu::Limits::default(),
        },
        None,
      )
      .await
      .expect("Failed to create device");

    // Load the shaders from disk
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
        Gpu::parse_shader(scene.lock().unwrap().reflection_limit).as_str(),
      )),
    });

    let (hdri, hdri_size) = Gpu::read_hdri();

    let hdri_texture_size = wgpu::Extent3d {
      width: hdri_size.0,
      height: hdri_size.1,
      depth_or_array_layers: 1,
    };

    let hdri_texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("hdri_texture"),
      size: hdri_texture_size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba32Float,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });

    queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: &hdri_texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      hdri.to_rgba32f().as_bytes(),
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: std::num::NonZeroU32::new(16 * hdri_size.0),
        rows_per_image: std::num::NonZeroU32::new(hdri_size.1),
      },
      hdri_texture_size,
    );

    let hdri_texture_view = hdri_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let hdri_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Nearest,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
    });

    let object_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: Scene::BUFFER_SIZE.0 as u64,
      mapped_at_creation: false,
      usage: wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC,
    });
    let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: Scene::BUFFER_SIZE.1 as u64,
      mapped_at_creation: false,
      usage: wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC,
    });
    let config_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: Scene::BUFFER_SIZE.2 as u64,
      mapped_at_creation: false,
      usage: wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: object_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: light_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 2,
          resource: config_buffer.as_entire_binding(),
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

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[swapchain_format.into()],
      }),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
    });

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: swapchain_format,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Mailbox,
    };

    surface.configure(&device, &config);

    Gpu {
      window,
      scene: scene.clone(),
      last_scene: scene.lock().unwrap().clone(),
      last_size: winit::dpi::PhysicalSize::new(0, 0),
      device,
      queue,
      surface,
      render_pipeline,
      bind_group,
      scene_buffers: SceneBuffers {
        objects: object_buffer,
        lights: light_buffer,
        config: config_buffer,
      },
      config,
    }
  }

  pub fn render(&mut self) {
    let size = self.window.inner_size();
    let scene = self.scene.lock().unwrap().clone();

    if (scene != self.last_scene) | (size != self.last_size) {
      self.scene_buffers.update(&scene, &self.queue, size);
    }

    self.last_scene = scene;
    self.last_size = size;

    let frame = self.surface
      .get_current_texture()
      .expect("Failed to acquire next swap chain texture");
    let view = frame
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder =
      self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
      let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
            store: true,
          },
        }],
        depth_stencil_attachment: None,
      });
      rpass.set_pipeline(&self.render_pipeline);
      rpass.set_bind_group(0, &self.bind_group, &[]);
      rpass.draw(0..6, 0..1);
    }

    self.queue.submit(Some(encoder.finish()));
    frame.present();
  }

  pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
    // Reconfigure the surface with the new size
    self.config.width = size.width;
    self.config.height = size.height;
    self.surface.configure(&self.device, &self.config);
    // On macos the window needs to be redrawn manually after resizing
    self.window.request_redraw();
  }

  pub fn request_render(&self) {
    self.window.request_redraw();
  }
}

pub async fn run(
  scene: Arc<Mutex<Scene>>,
  frame_times: Arc<Mutex<crate::History>>,
  fps_limit: f64,
) -> ! {
  let event_loop = EventLoop::<Window>::new_any_thread();
  let window = Window::new(&event_loop).unwrap();

  let mut last_time = crate::Time::now_millis();

  let mut gpu = Gpu::new(
    window,
    scene,
  ).await;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Poll;
    match event {
      Event::WindowEvent {
        event: WindowEvent::Resized(size),
        ..
      } => {
        gpu.resize(size);
      }
      Event::RedrawRequested(_) => {
        gpu.render();
      }
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => {
        *control_flow = ControlFlow::Exit;
      },
      _ => {
        let now = crate::Time::now_millis();
        let elapsed = now - last_time;

        if elapsed > 1000. / fps_limit {
          last_time = now;
          if let Ok(mut frame_times) = frame_times.try_lock() {
            frame_times.add(elapsed);
          }

          gpu.request_render();
        }
      }
    }
  });
}
