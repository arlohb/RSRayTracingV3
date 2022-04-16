mod connection;
pub use connection::Connection;
mod shaders;
pub use shaders::*;
pub mod shared_gpu;
pub use shared_gpu::SharedGpu;
pub mod render_texture;
pub use render_texture::RenderTexture;

use std::sync::{Arc, Mutex};
use crate::ray_tracer::Scene;

pub struct Gpu {
  shared_gpu: Arc<SharedGpu>,
  render_pipeline: wgpu::RenderPipeline,
  output_view: wgpu::TextureView,
  connection: Connection,
}

impl Gpu {
  pub fn new(
    shared_gpu: Arc<SharedGpu>,
    scene: Arc<Mutex<Scene>>,
  ) -> Gpu {
    let connection = Connection::new(scene.clone(), &shared_gpu.device, &shared_gpu.queue);

    let pipeline_layout = shared_gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[&connection.bind_group_layout],
      push_constant_ranges: &[],
    });

    let output_descriptor = wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width: 500,
        height: 500,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      usage: wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::TEXTURE_BINDING,
      label: None,
    };

    let output = shared_gpu.device.create_texture(&output_descriptor);
    let output_view = output.create_view(&Default::default());

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
            format: output_descriptor.format,
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
      output_view,
      connection,
    }
  }

  pub fn render(
    &mut self,
    egui_rpass: &mut egui_wgpu_backend::RenderPass,
    render_texture: &mut crate::gpu::RenderTexture,
  ) {
    self.connection.update_buffer(&self.shared_gpu.queue, winit::dpi::PhysicalSize::new(500, 500));

    let mut encoder =
      self.shared_gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
      let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[wgpu::RenderPassColorAttachment {
          view: &self.output_view,
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

    self.shared_gpu.queue.submit(Some(encoder.finish()));

    render_texture.update(
      &self.shared_gpu.device,
      egui_rpass,
      &self.output_view,
    );
  }

  pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
    let output_descriptor = wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      usage: wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::RENDER_ATTACHMENT,
      label: None,
    };

    let output = self.shared_gpu.device.create_texture(&output_descriptor);
    self.output_view = output.create_view(&Default::default());
  }
}
