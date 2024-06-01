use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use egui_winit::winit::{event::Event::*, event_loop::ControlFlow};
use std::{
    iter,
    sync::{Arc, Mutex},
};

use crate::gpu::{Connection, RenderTarget};
use crate::ray_tracer::Scene;

pub struct App {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,

    ui: crate::ui::Ui,

    render_pipeline: wgpu::RenderPipeline,
    previous_render_texture: wgpu::Texture,
    connection: Connection,
    render_target: RenderTarget,

    surface_config: wgpu::SurfaceConfiguration,

    egui_winit_state: egui_winit::State,
    egui_context: egui::Context,
    egui_renderer: Renderer,

    previous_frame_time: Option<f32>,
}

impl App {
    pub async fn new(
        window: &egui_winit::winit::window::Window,
        scene: Arc<Mutex<Scene>>,
        frame_times: Arc<Mutex<crate::utils::history::History>>,
        initial_render_size: (u32, u32),
    ) -> App {
        /* #region Initialize the GPU */

        let instance = wgpu::Instance::default();
        let surface = unsafe {
            instance
                .create_surface(window)
                .expect("Failed to create surface")
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
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

        /* #endregion */
        /* #region Initialize the surface */

        let size = window.inner_size();
        let surface_format = surface.get_capabilities(&adapter).formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            view_formats: vec![surface_format],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &surface_config);

        /* #endregion */
        /* #region Create the Egui UI */

        let egui_winit_state =
            egui_winit::State::new(egui::ViewportId::ROOT, window, Some(1.), None);
        let egui_context = egui::Context::default();

        let ui = crate::Ui::new(scene.clone(), frame_times);

        let egui_renderer = Renderer::new(&device, surface_format, None, 1);

        /* #endregion */
        /* #region Create the renderer */

        let render_target = crate::gpu::RenderTarget::new(&device, initial_render_size);
        let (previous_render_texture, previous_render_view) =
            RenderTarget::create_render_texture(&device, render_target.size);

        let connection = Connection::new(
            scene.clone(),
            &device,
            &queue,
            &previous_render_view,
            initial_render_size,
        )
        .await;

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&connection.bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &crate::gpu::vert_shader(&device),
                entry_point: "vs_main",
                buffers: &[Connection::vertex_buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &crate::gpu::frag_shader(&device),
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        /* #endregion */

        App {
            surface,
            device,
            queue,

            ui,

            render_pipeline,
            previous_render_texture,
            connection,
            render_target,

            surface_config,

            egui_winit_state,
            egui_context,
            egui_renderer,

            previous_frame_time: None,
        }
    }

    pub fn render(&mut self, window: &egui_winit::winit::window::Window) {
        /* #region Render the scene */

        self.connection
            .update_buffers(&self.queue, self.render_target.size);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target.render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.connection.bind_group, &[]);

            rpass.set_vertex_buffer(0, self.connection.vertex_buffer.slice(..));
            rpass.set_index_buffer(
                self.connection.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            rpass.draw_indexed(0..(Connection::INDICES_NUM as u32), 0, 0..1);
        }

        encoder.copy_texture_to_texture(
            self.render_target.render_texture.as_image_copy(),
            self.previous_render_texture.as_image_copy(),
            wgpu::Extent3d {
                width: self.render_target.size.0,
                height: self.render_target.size.1,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        self.render_target
            .update(&self.device, &mut self.egui_renderer);

        /* #endregion */
        /* #region Render the UI */

        let output_frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                println!("Dropped frame with error: {}", e);
                return;
            }
        };
        let output_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Begin to draw the UI frame.
        let egui_start = crate::utils::time::now_millis();
        let input = self.egui_winit_state.take_egui_input(window);
        self.egui_context.begin_frame(input);

        self.ui
            .update(&self.egui_context, &mut self.render_target, &self.device);

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let output = self.egui_context.end_frame();
        let paint_jobs = self
            .egui_context
            .tessellate(output.shapes, self.egui_context.pixels_per_point());

        let frame_time = (crate::utils::time::now_millis() - egui_start) / 1000.;
        self.previous_frame_time = Some(frame_time as f32);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.surface_config.width, self.surface_config.height],
            pixels_per_point: 1.,
        };

        // Update textures
        for (texture_id, image_delta) in output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, texture_id, &image_delta);
        }

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            // Record all render passes.
            self.egui_renderer
                .render(&mut rpass, &paint_jobs, &screen_descriptor);
        }

        // Free textures
        for texture_id in output.textures_delta.free {
            self.egui_renderer.free_texture(&texture_id);
        }

        // Submit the commands.
        self.queue.submit(iter::once(encoder.finish()));

        // Redraw egui
        output_frame.present();

        /* #endregion */
    }
}

pub async fn run(
    event_loop: egui_winit::winit::event_loop::EventLoop<()>,
    window: egui_winit::winit::window::Window,
    scene: Arc<Mutex<Scene>>,
    frame_times: Arc<Mutex<crate::utils::history::History>>,
    fps_limit: f64,
    initial_render_size: (u32, u32),
) {
    let mut last_time = crate::utils::time::now_millis();

    let mut app = App::new(
        &window,
        scene.clone(),
        frame_times.clone(),
        initial_render_size,
    )
    .await;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            RedrawRequested(..) => {
                let now = crate::utils::time::now_millis();
                let elapsed = now - last_time;
                if elapsed > 1000. / fps_limit {
                    last_time = now;
                    if let Ok(mut frame_times) = frame_times.try_lock() {
                        frame_times.add(elapsed);
                    }
                    app.render(&window);
                }
            }
            MainEventsCleared => {
                window.request_redraw();
            }
            WindowEvent { event, .. } => match event {
                egui_winit::winit::event::WindowEvent::Resized(size) => {
                    app.surface_config.width = size.width;
                    app.surface_config.height = size.height;
                    app.surface.configure(&app.device, &app.surface_config);
                }
                egui_winit::winit::event::WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                event => {
                    // Pass the winit events to the platform integration.
                    let _ = app
                        .egui_winit_state
                        .on_window_event(&app.egui_context, &event);
                }
            },
            _ => (),
        }
    });
}
