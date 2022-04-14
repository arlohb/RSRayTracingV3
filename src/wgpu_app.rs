use std::{
  time::Instant,
  iter,
  sync::{Arc, Mutex},
};
use egui_wgpu_backend::{
  RenderPass,
  ScreenDescriptor,
};
use winit::{
  event::Event::*,
  event_loop::ControlFlow,
};

use crate::ray_tracer::Scene;

/// A custom event type for the winit app.
enum Event {
  RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
pub struct RepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<Event>>);

impl epi::backend::RepaintSignal for RepaintSignal {
  fn request_repaint(&self) {
    self.0.lock().unwrap().send_event(Event::RequestRedraw).ok();
  }
}

pub struct WgpuApp {
  pub window: winit::window::Window,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub surface: wgpu::Surface,
  pub surface_config: wgpu::SurfaceConfiguration,
  pub surface_format: wgpu::TextureFormat,
  pub state: egui_winit::State,
  pub context: egui::Context,
  pub egui_rpass: RenderPass,
  pub app: Box<dyn epi::App>,
  pub previous_frame_time: Option<f32>,
  pub repaint_signal: Arc<RepaintSignal>,
}

impl WgpuApp {
  pub async fn new(
    window: winit::window::Window,
    app: Box<dyn epi::App>,
    repaint_signal: Arc<RepaintSignal>,
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

    let state = egui_winit::State::new(4096, &window);
    let context = egui::Context::default();

    // We use the egui_wgpu_backend crate as the render backend.
    let egui_rpass = RenderPass::new(&device, surface_format, 1);

    WgpuApp {
      window,
      device,
      queue,
      surface,
      surface_config,
      surface_format,
      state,
      context,
      egui_rpass,
      app,
      previous_frame_time: None,
      repaint_signal,
    }
  }

  pub fn render(&mut self) {
    let output_frame = match self.surface.get_current_texture() {
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
    let input = self.state.take_egui_input(&self.window);
    self.context.begin_frame(input);
    let app_output = epi::backend::AppOutput::default();

    let frame =  epi::Frame::new(epi::backend::FrameData {
      info: epi::IntegrationInfo {
        name: "egui_example",
        web_info: None,
        cpu_usage: self.previous_frame_time,
        native_pixels_per_point: Some(self.window.scale_factor() as _),
        prefer_dark_mode: None,
      },
      output: app_output,
      repaint_signal: self.repaint_signal.clone(),
    });

    // Draw the demo application.
    self.app.update(&self.context, &frame);

    // End the UI frame. We could now handle the output and draw the UI with the backend.
    let output = self.context.end_frame();
    let paint_jobs = self.context.tessellate(output.shapes);

    let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
    self.previous_frame_time = Some(frame_time);

    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("encoder"),
    });

    // Upload all resources for the GPU.
    let screen_descriptor = ScreenDescriptor {
      physical_width: self.surface_config.width,
      physical_height: self.surface_config.height,
      scale_factor: self.window.scale_factor() as f32,
    };

    self.egui_rpass.add_textures(&self.device, &self.queue, &output.textures_delta).expect("Failed to add egui textures");
    self.egui_rpass.remove_textures(output.textures_delta).expect("Failed to remove egui textures");
    self.egui_rpass.update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

    // Record all render passes.
    self.egui_rpass
      .execute(
        &mut encoder,
        &output_view,
        &paint_jobs,
        &screen_descriptor,
        Some(wgpu::Color::BLACK),
      )
      .expect("Failed to execute render pass");
    // Submit the commands.
    self.queue.submit(iter::once(encoder.finish()));

    // Redraw egui
    output_frame.present();
  }
}

pub async fn run(
  app: Box<dyn epi::App>,
  scene: Arc<Mutex<Scene>>,
  frame_times: Arc<Mutex<crate::History>>,
  fps_limit: f64,
) {
  let event_loop = winit::event_loop::EventLoop::with_user_event();

  let mut last_time = crate::Time::now_millis();

  let mut gpu = crate::gpu::Gpu::new(
    winit::window::Window::new(&event_loop).unwrap(),
    scene,
  ).await;

  let mut wgpu_app = crate::wgpu_app::WgpuApp::new(
    winit::window::WindowBuilder::new()
      .with_decorations(true)
      .with_resizable(true)
      .with_transparent(false)
      .with_title("egui-wgpu_winit example")
      .with_inner_size(winit::dpi::PhysicalSize {
        width: 800u32,
        height: 700u32,
      })
      .build(&event_loop)
      .expect("Failed to create window"),
    app,
    Arc::new(RepaintSignal(std::sync::Mutex::new(
      event_loop.create_proxy(),
    ))),
  ).await;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Poll;
    match event {
      RedrawRequested(window_id) => {
        if window_id == wgpu_app.window.id() {
          let now = crate::Time::now_millis();
          let elapsed = now - last_time;
          if elapsed > 1000. / fps_limit {
            last_time = now;
            if let Ok(mut frame_times) = frame_times.try_lock() {
              frame_times.add(elapsed);
            }

            wgpu_app.render();
            gpu.render();
          }
        }
      }
      MainEventsCleared | UserEvent(Event::RequestRedraw) => {
        wgpu_app.window.request_redraw();
      }
      WindowEvent { window_id, event, .. } => match event {
        winit::event::WindowEvent::Resized(size) => {
          if window_id == wgpu_app.window.id() {
            wgpu_app.surface_config.width = size.width;
            wgpu_app.surface_config.height = size.height;
            wgpu_app.surface.configure(&wgpu_app.device, &wgpu_app.surface_config);
          } else {
            gpu.resize(size);
          }
        }
        winit::event::WindowEvent::CloseRequested => {
          *control_flow = ControlFlow::Exit;
        }
        event => {
          if window_id == wgpu_app.window.id() {
            // Pass the winit events to the platform integration.
            wgpu_app.state.on_event(&wgpu_app.context, &event);
          }
        }
      },
      _ => (),
    }
  });
}
