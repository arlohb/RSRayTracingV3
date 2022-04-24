mod connection;
pub use connection::Connection;
mod shaders;
pub use shaders::*;
pub mod shared_gpu;
pub use shared_gpu::SharedGpu;
pub mod render_target;
pub use render_target::RenderTarget;

use std::sync::{Arc, Mutex};
use crate::ray_tracer::Scene;

pub struct Gpu {
  shared_gpu: Arc<SharedGpu>,
  render_pipeline: wgpu::RenderPipeline,
  previous_render_texture: wgpu::Texture,
  connection: Connection,
}

impl Gpu {
  pub fn new(
    shared_gpu: Arc<SharedGpu>,
    scene: Arc<Mutex<Scene>>,
    render_target: &RenderTarget,
  ) -> Gpu {
    let (previous_render_texture, previous_render_view) = RenderTarget::create_render_texture(
      &shared_gpu,
      render_target.size,
    );

    let connection = Connection::new(scene.clone(), &shared_gpu.device, &shared_gpu.queue, &previous_render_view);

    let pipeline_layout = shared_gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[&connection.bind_group_layout],
      push_constant_ranges: &[],
    });

    let render_pipeline = shared_gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &crate::gpu::vert_shader(&shared_gpu.device),
        entry_point: "vs_main",
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &crate::gpu::frag_shader(&shared_gpu.device, scene.lock().unwrap().reflection_limit),
        entry_point: "fs_main",
        targets: &[
          wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
          }
        ],
      }),
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
    });

    Gpu {
      shared_gpu,
      render_pipeline,
      previous_render_texture,
      connection,
    }
  }

  pub fn render(
    &mut self,
    egui_rpass: &mut egui_wgpu_backend::RenderPass,
    render_target: &mut RenderTarget,
  ) {
    self.connection.update_buffers(&self.shared_gpu.queue, render_target.size);

    let mut encoder =
      self.shared_gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });    

    {
      let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[wgpu::RenderPassColorAttachment {
          view: &render_target.render_view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: true,
          },
        }],
        depth_stencil_attachment: None,
      });
      rpass.set_pipeline(&self.render_pipeline);
      rpass.set_bind_group(0, &self.connection.bind_group, &[]);
      rpass.draw(0..6, 0..1);
    }

    encoder.copy_texture_to_texture(
      render_target.render_texture.as_image_copy(),
      self.previous_render_texture.as_image_copy(),
      wgpu::Extent3d {
        width: render_target.size.0,
        height: render_target.size.1,
        depth_or_array_layers: 1,
      },
    );

    self.shared_gpu.queue.submit(Some(encoder.finish()));

    render_target.update(
      &self.shared_gpu.device,
      egui_rpass,
    );
  }
}
