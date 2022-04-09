use std::{
  borrow::Cow,
  sync::{Arc, Mutex},
};
use wgpu::util::DeviceExt;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::Window,
};

use crate::ray_tracer::Scene;

pub async fn run(event_loop: EventLoop<()>, window: Window, scene: Arc<Mutex<Scene>>) {
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
        features: wgpu::Features::empty(),
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
      (include_str!("vert.wgsl").to_string() + include_str!("frag.wgsl")).as_str(),
    )),
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
    ],
  });

  let data = [window.inner_size().width as u32, window.inner_size().height as u32];

  let input_bytes : &[u8] = bytemuck::bytes_of(&data);
  let input_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: None,
    contents: input_bytes,
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
        resource: input_buf.as_entire_binding(),
      },
    ],
  });

  let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: None,
    bind_group_layouts: &[
      &bind_group_layout,
    ],
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

  let mut config = wgpu::SurfaceConfiguration {
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
    format: swapchain_format,
    width: size.width,
    height: size.height,
    present_mode: wgpu::PresentMode::Mailbox,
  };

  surface.configure(&device, &config);

  event_loop.run(move |event, _, control_flow| {
    // Have the closure take ownership of the resources.
    // `event_loop.run` never returns, therefore we must do this to ensure
    // the resources are properly cleaned up.
    let _ = (&instance, &adapter, &shader, &pipeline_layout);

    *control_flow = ControlFlow::Wait;
    match event {
      Event::WindowEvent {
        event: WindowEvent::Resized(size),
        ..
      } => {
        // Reconfigure the surface with the new size
        config.width = size.width;
        config.height = size.height;
        surface.configure(&device, &config);
        // On macos the window needs to be redrawn manually after resizing
        window.request_redraw();
      }
      Event::RedrawRequested(_) => {
        let data = [window.inner_size().width as u32, window.inner_size().height as u32];
        queue.write_buffer(&input_buf, 0, bytemuck::bytes_of(&data));

        let frame = surface
          .get_current_texture()
          .expect("Failed to acquire next swap chain texture");
        let view = frame
          .texture
          .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
          device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
          rpass.set_pipeline(&render_pipeline);
          rpass.set_bind_group(0, &bind_group, &[]);
          rpass.draw(0..6, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
      }
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => *control_flow = ControlFlow::Exit,
      _ => {}
    }
  });
}
