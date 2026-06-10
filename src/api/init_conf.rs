use wgpu::{PresentMode, TextureFormat};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Clone)]
pub struct CameraConfig {
	pub position: [f32; 3],
	pub look_at: [f32; 3],
	pub up: [f32; 3],
	pub fov: f32,
	pub near: f32,
	pub far: f32,
}

impl Default for CameraConfig {
	fn default() -> Self {
		Self {
			position: [800.0, 800.0, 800.0],
			look_at: [0.0, 0.0, 0.0],
			up: [0.0, 1.0, 0.0],
			fov: 60.0_f32.to_radians(),
			near: 1.0,
			far: 10000.0,
		}
	}
}

#[derive(Clone)]
pub struct InputConfig {
	pub orbit_button: MouseButton,
	pub orbit_sensitivity: f32,
	pub zoom_sensitivity: f32,
	pub pan_speed: f32,
	pub forward_key: KeyCode,
	pub backward_key: KeyCode,
	pub left_key: KeyCode,
	pub right_key: KeyCode,
	pub pitch_clamp_min: f32,
	pub pitch_clamp_max: f32,
	pub distance_clamp_min: f32,
	pub distance_clamp_max: f32,
}

impl Default for InputConfig {
	fn default() -> Self {
		Self {
			orbit_button: MouseButton::Right,
			orbit_sensitivity: 0.005,
			zoom_sensitivity: 3.0,
			pan_speed: 300.0,
			forward_key: KeyCode::KeyW,
			backward_key: KeyCode::KeyS,
			left_key: KeyCode::KeyA,
			right_key: KeyCode::KeyD,
			pitch_clamp_min: -1.5,
			pitch_clamp_max: 1.5,
			distance_clamp_min: 1.0,
			distance_clamp_max: 10000.0,
		}
	}
}

#[derive(Clone)]
pub struct RenderConfig {
	pub clear_color: [f64; 4],
	pub present_mode: PresentMode,
	pub depth_format: TextureFormat,
}

impl Default for RenderConfig {
	fn default() -> Self {
		Self {
			clear_color: [0.1, 0.2, 0.3, 1.0],
			present_mode: PresentMode::Fifo,
			depth_format: TextureFormat::Depth32Float,
		}
	}
}

#[derive(Clone)]
pub struct WindowConfig {
	pub title: &'static str,
}

impl Default for WindowConfig {
	fn default() -> Self {
		Self { title: "IceSpar" }
	}
}

#[derive(Clone, Default)]
pub struct InitConfig {
	pub camera: CameraConfig,
	pub input: InputConfig,
	pub render: RenderConfig,
	pub window: WindowConfig,
}
