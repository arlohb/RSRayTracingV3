use wgpu::util::DeviceExt;

pub struct Gpu {
  device: wgpu::Device,
  queue: wgpu::Queue,
  shader: wgpu::ShaderModule,
}

impl Gpu {
  pub async fn new() -> Self {
    // so that wgpu doesn't silently fail
    env_logger::init();

    // create the instance
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        
    // create a general connection to the GPU
    let adapter = instance.request_adapter(&Default::default()).await.unwrap();

    // create a specific connection to the GPU
    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
          limits: Default::default(),
        },
        None,
      )
      .await
      .unwrap();

    // load the shader
    let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    Gpu {
      device,
      queue,
      shader,
    }
  }

  pub async fn run(&self, input: &[f32; 2]) -> [f32; 2] {
    // deal with the input

    let input_bytes : &[u8] = bytemuck::bytes_of(input);
    let input_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: input_bytes,
      usage: wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC,
    });
    let output_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
      label: None,
      size: input_bytes.len() as u64,
      usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    // bind group stuff

    let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: None,
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::COMPUTE,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ],
    });

    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: input_buf.as_entire_binding(),
        },
      ],
    });

    let compute_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });
    let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
      label: None,
      layout: Some(&compute_pipeline_layout),
      module: &self.shader,
      entry_point: "main",
    });

    let mut encoder = self.device.create_command_encoder(&Default::default());
    {
      let mut cpass = encoder.begin_compute_pass(&Default::default());
      cpass.set_pipeline(&pipeline);
      cpass.set_bind_group(0, &bind_group, &[]);
      cpass.dispatch(input.len() as u32, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&input_buf, 0, &output_buf, 0, input_bytes.len() as u64);

    // submits encoder for processing
    self.queue.submit(Some(encoder.finish()));

    let buf_slice = output_buf.slice(..);
    let buf_future = buf_slice.map_async(wgpu::MapMode::Read);

    self.device.poll(wgpu::Maintain::Wait);

    // waits until buf_future can be read from
    if buf_future.await.is_ok() {
      let data_raw = &*buf_slice.get_mapped_range();
      let data : &[f32] = bytemuck::cast_slice(data_raw);
      [data[0], data[1]]
    } else {
      panic!("Failed to read buffer");
    }
  }
}
