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
use crate::gpu::SharedGpu;

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
  pub shared_gpu: Arc<SharedGpu>,
  pub surface_config: wgpu::SurfaceConfiguration,
  pub surface_format: wgpu::TextureFormat,
  pub state: egui_winit::State,
  pub context: egui::Context,
  pub egui_rpass: RenderPass,
  pub app: crate::app::App,
  pub previous_frame_time: Option<f32>,
  pub repaint_signal: Arc<RepaintSignal>,
  pub render_texture: crate::gpu::RenderTexture,
}

impl WgpuApp {
  pub fn new(
    shared_gpu: Arc<SharedGpu>,
    window: &winit::window::Window,
    repaint_signal: Arc<RepaintSignal>,
    scene: Arc<Mutex<Scene>>,
    frame_times: Arc<Mutex<crate::History>>,
  ) -> WgpuApp {
    let size = window.inner_size();
    let surface_format = shared_gpu.surface.get_preferred_format(&shared_gpu.adapter).expect("Surface format not supported");
    let surface_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width as u32,
      height: size.height as u32,
      present_mode: wgpu::PresentMode::Mailbox,
    };
    shared_gpu.surface.configure(&shared_gpu.device, &surface_config);

    let state = egui_winit::State::new(4096, window);
    let context = egui::Context::default();

    let render_texture = crate::gpu::RenderTexture::new();

    let app = crate::App::new(scene, frame_times);

    // We use the egui_wgpu_backend crate as the render backend.
    let egui_rpass = RenderPass::new(&shared_gpu.device, surface_format, 1);

    WgpuApp {
      shared_gpu,
      surface_config,
      surface_format,
      state,
      context,
      egui_rpass,
      app,
      previous_frame_time: None,
      repaint_signal,
      render_texture,
    }
  }

  pub fn render(
    &mut self,
    window: &winit::window::Window,
    render_texture: crate::gpu::RenderTexture,
  ) {
    let output_frame = match self.shared_gpu.surface.get_current_texture() {
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
    let input = self.state.take_egui_input(window);
    self.context.begin_frame(input);
    let app_output = epi::backend::AppOutput::default();

    let frame =  epi::Frame::new(epi::backend::FrameData {
      info: epi::IntegrationInfo {
        name: "egui_example",
        web_info: None,
        cpu_usage: self.previous_frame_time,
        native_pixels_per_point: Some(window.scale_factor() as _),
        prefer_dark_mode: None,
      },
      output: app_output,
      repaint_signal: self.repaint_signal.clone(),
    });

    // Draw the demo application.
    self.app.update(&self.context, &frame, render_texture);

    // End the UI frame. We could now handle the output and draw the UI with the backend.
    let output = self.context.end_frame();
    let paint_jobs = self.context.tessellate(output.shapes);

    let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
    self.previous_frame_time = Some(frame_time);

    let mut encoder = self.shared_gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("encoder"),
    });

    // Upload all resources for the GPU.
    let screen_descriptor = ScreenDescriptor {
      physical_width: self.surface_config.width,
      physical_height: self.surface_config.height,
      scale_factor: window.scale_factor() as f32,
    };

    self.egui_rpass.add_textures(&self.shared_gpu.device, &self.shared_gpu.queue, &output.textures_delta).expect("Failed to add egui textures");
    self.egui_rpass.remove_textures(output.textures_delta).expect("Failed to remove egui textures");
    self.egui_rpass.update_buffers(&self.shared_gpu.device, &self.shared_gpu.queue, &paint_jobs, &screen_descriptor);

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
    self.shared_gpu.queue.submit(iter::once(encoder.finish()));

    // Redraw egui
    output_frame.present();
  }
}

pub fn run(
  scene: Arc<Mutex<Scene>>,
  frame_times: Arc<Mutex<crate::History>>,
  fps_limit: f64,
) {
  let event_loop = winit::event_loop::EventLoop::with_user_event();

  let mut last_time = crate::Time::now_millis();

  let window = winit::window::WindowBuilder::new()
    .with_decorations(true)
    .with_resizable(true)
    .with_transparent(false)
    .with_title("egui-wgpu_winit example")
    .with_inner_size(winit::dpi::PhysicalSize {
      width: 1200u32,
      height: 800u32,
    })
    .build(&event_loop)
    .expect("Failed to create window");

  let shared_gpu = Arc::new(SharedGpu::new(&window));

  let mut gpu = crate::gpu::Gpu::new(shared_gpu.clone(), scene.clone());

  let mut wgpu_app = crate::wgpu_app::WgpuApp::new(
    shared_gpu.clone(),
    &window,
    Arc::new(RepaintSignal(std::sync::Mutex::new(
      event_loop.create_proxy(),
    ))),
    scene.clone(),
    frame_times.clone(),
  );

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Poll;
    match event {
      RedrawRequested(window_id) => {
        if window_id == window.id() {
          let now = crate::Time::now_millis();
          let elapsed = now - last_time;
          if elapsed > 1000. / fps_limit {
            last_time = now;
            if let Ok(mut frame_times) = frame_times.try_lock() {
              frame_times.add(elapsed);
            }

            gpu.render(&mut wgpu_app.egui_rpass, &mut wgpu_app.render_texture);
            wgpu_app.render(&window, wgpu_app.render_texture);
          }
        }
      }
      MainEventsCleared | UserEvent(Event::RequestRedraw) => {
        window.request_redraw();
      }
      WindowEvent { window_id, event, .. } => match event {
        winit::event::WindowEvent::Resized(size) => {
          if window_id == window.id() {
            wgpu_app.surface_config.width = size.width;
            wgpu_app.surface_config.height = size.height;
            shared_gpu.surface.configure(&shared_gpu.device, &wgpu_app.surface_config);
          } else {
            gpu.resize(size);
          }
        }
        winit::event::WindowEvent::CloseRequested => {
          *control_flow = ControlFlow::Exit;
        }
        event => {
          if window_id == window.id() {
            // Pass the winit events to the platform integration.
            wgpu_app.state.on_event(&wgpu_app.context, &event);
          }
        }
      },
      _ => (),
    }
  });
}
