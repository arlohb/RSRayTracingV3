use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::winit::{
    event::Event::{AboutToWait, WindowEvent},
    event_loop::ControlFlow,
};
use std::{iter, sync::Arc};

use crate::ray_tracer::Scene;
use crate::{
    gpu::{Connection, RenderTarget},
    utils::history::History,
};

pub struct App {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,

    ui: crate::ui::Ui,
    scene: Scene,
    frame_times: History,

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
        window: Arc<egui_winit::winit::window::Window>,
        scene: Scene,
        frame_times: History,
        initial_render_size: (u32, u32),
    ) -> Self {
        /* #region Initialize the GPU */

        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
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
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        /* #endregion */
        /* #region Create the Egui UI */

        let egui_context = egui::Context::default();
        let egui_winit_state = egui_winit::State::new(
            egui_context.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(1.),
            None,
        );

        let ui = crate::Ui::new();
        let egui_renderer = Renderer::new(&device, surface_format, None, 1);

        /* #endregion */
        /* #region Create the renderer */

        let render_target = crate::gpu::RenderTarget::new(&device, initial_render_size);
        let (previous_render_texture, previous_render_view) =
            RenderTarget::create_render_texture(&device, render_target.size);

        let connection = Connection::new(&scene, &device, &queue, &previous_render_view);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&connection.bind_group_layout],
            ..Default::default()
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
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        /* #endregion */

        Self {
            surface,
            device,
            queue,

            ui,
            scene,
            frame_times,

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
        puffin::profile_function!();

        /* #region Render the scene */

        self.connection
            .update_buffers(&self.queue, self.render_target.size, &self.scene);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target.render_view,
                    resolve_target: None,
                    ops: wgpu::Operations::default(),
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
                println!("Dropped frame with error: {e}");
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

        self.ui.update(
            &self.egui_context,
            &mut self.render_target,
            &self.device,
            &mut self.scene,
            &self.frame_times,
        );

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
                    ops: wgpu::Operations::default(),
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

pub fn run(
    event_loop: egui_winit::winit::event_loop::EventLoop<()>,
    window: Arc<egui_winit::winit::window::Window>,
    scene: Scene,
    frame_times: History,
    fps_limit: f64,
    initial_render_size: (u32, u32),
) {
    let mut last_time = crate::utils::time::now_millis();

    let mut app = pollster::block_on(App::new(
        window.clone(),
        scene,
        frame_times,
        initial_render_size,
    ));

    event_loop
        .run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);
            match event {
                WindowEvent { event, .. } => match event {
                    egui_winit::winit::event::WindowEvent::RedrawRequested => {
                        puffin::GlobalProfiler::lock().new_frame();

                        let now = crate::utils::time::now_millis();
                        let elapsed = now - last_time;
                        if elapsed > 1000. / fps_limit {
                            last_time = now;
                            app.frame_times.add(elapsed);
                            app.render(&window);
                        }
                    }
                    egui_winit::winit::event::WindowEvent::Resized(size) => {
                        app.surface_config.width = size.width;
                        app.surface_config.height = size.height;
                        app.surface.configure(&app.device, &app.surface_config);
                    }
                    egui_winit::winit::event::WindowEvent::CloseRequested => {
                        window_target.exit();
                    }
                    event => {
                        // Pass the winit events to the platform integration.
                        let _ = app.egui_winit_state.on_window_event(&window, &event);
                    }
                },
                AboutToWait => window.request_redraw(),
                _ => (),
            }
        })
        .expect("Event loop failed to run");
}
