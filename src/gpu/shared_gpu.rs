pub struct SharedGpu {
  pub instance: wgpu::Instance,
  pub surface: wgpu::Surface,
  pub adapter: wgpu::Adapter,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
}

impl SharedGpu {
  pub async fn new(window: &winit::window::Window) -> SharedGpu {
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(window) };
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
      
      SharedGpu {
      instance,
      surface,
      adapter,
      device,
      queue,
    }
  }
}
