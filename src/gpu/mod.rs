mod connection;
pub use connection::Connection;
mod shaders;
pub use shaders::*;

use std::sync::{Arc, Mutex};
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::Window,
  platform::unix::EventLoopExtUnix,
};
use crate::ray_tracer::Scene;

pub struct Gpu {
  window: Window,
  device: wgpu::Device,
  queue: wgpu::Queue,
  surface: wgpu::Surface,
  config: wgpu::SurfaceConfiguration,
  render_pipeline: wgpu::RenderPipeline,
  connection: Connection,
}

impl Gpu {
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

    let connection = Connection::new(scene.clone(), &device, &queue);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[&connection.bind_group_layout],
      push_constant_ranges: &[],
    });

    let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &crate::gpu::vert_shader(&device),
        entry_point: "vs_main",
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &crate::gpu::frag_shader(&device, scene.lock().unwrap().reflection_limit),
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
      device,
      queue,
      surface,
      render_pipeline,
      connection,
      config,
    }
  }

  pub fn render(&mut self) {
    self.connection.update_buffer(&self.queue, self.window.inner_size());

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
      rpass.set_bind_group(0, &self.connection.bind_group, &[]);
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
