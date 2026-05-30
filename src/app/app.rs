use std::sync::Arc;

use pollster::block_on;
use wgpu::{
	Backends, Device, DeviceDescriptor, Extent3d, Features, Instance, Limits, Queue,
	RequestAdapterOptionsBase, Surface, SurfaceConfiguration, Texture, TextureDimension,
	TextureFormat, TextureUsages,
};
use winit::event_loop::EventLoop;
use winit::window::Window;

pub struct App {
	pub window: Arc<Window>,
	pub surface: Surface<'static>,
	pub device: Device,
	pub queue: Queue,
	pub surface_conf: SurfaceConfiguration,

	pub depth_texture: Option<Texture>,
}
impl App {
	pub fn instance(event_loop: &EventLoop<()>) -> Self {
		let window = Arc::new(Window::new(event_loop).unwrap());
		let app_init = block_on(async move {
			let instance = Instance::new(wgpu::InstanceDescriptor {
				backends: Backends::all(),
				..Default::default()
			});
			let surface = { instance.create_surface(window.clone()) }.unwrap();

			let adapter = instance
				.request_adapter(&RequestAdapterOptionsBase {
					power_preference: wgpu::PowerPreference::HighPerformance,
					force_fallback_adapter: false,
					compatible_surface: Some(&surface),
				})
				.await
				.unwrap();

			let (device, queue) = adapter
				.request_device(
					&DeviceDescriptor {
						label: Some("just a device"),
						required_features: Features::empty(),
						required_limits: Limits::default(),
					},
					None,
				)
				.await
				.unwrap();

			let sur_cap = surface.get_capabilities(&adapter);
			let mut size = window.inner_size();
			size.width = size.width.max(1);
			size.height = size.height.max(1);

			let surface_conf = SurfaceConfiguration {
				usage: TextureUsages::RENDER_ATTACHMENT,
				format: sur_cap.formats[0],
				width: size.width,
				height: size.height,
				present_mode: wgpu::PresentMode::Fifo,
				desired_maximum_frame_latency: 2,
				alpha_mode: sur_cap.alpha_modes[0],
				view_formats: vec![],
			};

			surface.configure(&device, &surface_conf);

			(window, surface, device, queue, surface_conf)
		});
		App {
			window: app_init.0,
			surface: app_init.1,
			device: app_init.2,
			queue: app_init.3,
			surface_conf: app_init.4,

			depth_texture: None,
		}
	}
	pub fn enable_depth_test(&mut self) {
		let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Depth Texture"),
			size: Extent3d {
				width: self.surface_conf.width,
				height: self.surface_conf.height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format: TextureFormat::Depth32Float,
			usage: TextureUsages::RENDER_ATTACHMENT,
			view_formats: &[],
		});
		self.depth_texture = Some(depth_texture);
	}
}
