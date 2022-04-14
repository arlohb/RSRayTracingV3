use std::{
  time::Instant,
  iter,
};
use egui_wgpu_backend::{
  RenderPass,
  ScreenDescriptor,
};
use winit::{
  event::Event::*,
  event_loop::ControlFlow,
};

/// A custom event type for the winit app.
enum Event {
  RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct ExampleRepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<Event>>);

impl epi::backend::RepaintSignal for ExampleRepaintSignal {
  fn request_repaint(&self) {
    self.0.lock().unwrap().send_event(Event::RequestRedraw).ok();
  }
}

pub struct WgpuApp {
  window: winit::window::Window,
  device: wgpu::Device,
  queue: wgpu::Queue,
  surface: wgpu::Surface,
  surface_config: wgpu::SurfaceConfiguration,
  surface_format: wgpu::TextureFormat,
}

impl WgpuApp {
  pub async fn new(
    window: winit::window::Window,
  ) -> WgpuApp {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::HighPerformance,
      compatible_surface: Some(&surface),
      force_fallback_adapter: false,
    }).await.expect("Failed to get video adapter");

    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        features: wgpu::Features::default(),
        limits: wgpu::Limits::default(),
        label: None,
      },
      None,
    ).await.expect("Failed to create device");

    let size = window.inner_size();
    let surface_format = surface.get_preferred_format(&adapter).expect("Surface format not supported");
    let surface_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width as u32,
      height: size.height as u32,
      present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &surface_config);

    WgpuApp {
      window,
      device,
      queue,
      surface,
      surface_config,
      surface_format,
    }
  }
}

pub async fn run(app: Box<dyn epi::App>) {
  let mut app = app;

  let event_loop = winit::event_loop::EventLoop::with_user_event();
  let window = winit::window::WindowBuilder::new()
    .with_decorations(true)
    .with_resizable(true)
    .with_transparent(false)
    .with_title("egui-wgpu_winit example")
    .with_inner_size(winit::dpi::PhysicalSize {
      width: 800u32,
      height: 700u32,
    })
    .build(&event_loop)
    .expect("Failed to create window");

  let mut wgpu_app = crate::wgpu_app::WgpuApp::new(window).await;

  let repaint_signal = std::sync::Arc::new(ExampleRepaintSignal(std::sync::Mutex::new(
    event_loop.create_proxy(),
  )));

  let mut state = egui_winit::State::new(4096, &wgpu_app.window);
  let context = egui::Context::default();

  // We use the egui_wgpu_backend crate as the render backend.
  let mut egui_rpass = RenderPass::new(&wgpu_app.device, wgpu_app.surface_format, 1);

  let mut previous_frame_time = None;
  event_loop.run(move |event, _, control_flow| {
    match event {
      RedrawRequested(..) => {
        let output_frame = match wgpu_app.surface.get_current_texture() {
          Ok(frame) => frame,
          Err(e) => {
            eprintln!("Dropped frame with error: {}", e);
            return;
          }
        };
        let output_view = output_frame
          .texture
          .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        let egui_start = Instant::now();
        let input = state.take_egui_input(&wgpu_app.window);
        context.begin_frame(input);
        let app_output = epi::backend::AppOutput::default();

        let frame =  epi::Frame::new(epi::backend::FrameData {
          info: epi::IntegrationInfo {
            name: "egui_example",
            web_info: None,
            cpu_usage: previous_frame_time,
            native_pixels_per_point: Some(wgpu_app.window.scale_factor() as _),
            prefer_dark_mode: None,
          },
          output: app_output,
          repaint_signal: repaint_signal.clone(),
        });

        // Draw the demo application.
        app.update(&context, &frame);

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let output = context.end_frame();
        let paint_jobs = context.tessellate(output.shapes);

        let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
        previous_frame_time = Some(frame_time);

        let mut encoder = wgpu_app.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("encoder"),
        });

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
          physical_width: wgpu_app.surface_config.width,
          physical_height: wgpu_app.surface_config.height,
          scale_factor: wgpu_app.window.scale_factor() as f32,
        };

        egui_rpass.add_textures(&wgpu_app.device, &wgpu_app.queue, &output.textures_delta).expect("Failed to add egui textures");
        egui_rpass.remove_textures(output.textures_delta).expect("Failed to remove egui textures");
        egui_rpass.update_buffers(&wgpu_app.device, &wgpu_app.queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        egui_rpass
          .execute(
            &mut encoder,
            &output_view,
            &paint_jobs,
            &screen_descriptor,
            Some(wgpu::Color::BLACK),
          )
          .expect("Failed to execute render pass");
        // Submit the commands.
        wgpu_app.queue.submit(iter::once(encoder.finish()));

        // Redraw egui
        output_frame.present();
      }
      MainEventsCleared | UserEvent(Event::RequestRedraw) => {
        wgpu_app.window.request_redraw();
      }
      WindowEvent { event, .. } => match event {
        winit::event::WindowEvent::Resized(size) => {
          wgpu_app.surface_config.width = size.width;
          wgpu_app.surface_config.height = size.height;
          wgpu_app.surface.configure(&wgpu_app.device, &wgpu_app.surface_config);
        }
        winit::event::WindowEvent::CloseRequested => {
          *control_flow = ControlFlow::Exit;
        }
        event => {
          // Pass the winit events to the platform integration.
          state.on_event(&context, &event);
        }
      },
      _ => (),
    }
  });
}
