use winit::dpi::PhysicalSize;

use crate::app::app::App;

pub struct WindowHandle {
	pub is_need_resize: bool,
	pub current_size: PhysicalSize<u32>,
}
impl WindowHandle {
	pub fn resize_window(&mut self, app: &mut App) {
		if self.is_need_resize {
			app.surface_conf.width = self.current_size.width;
			app.surface_conf.height = self.current_size.height;
			app.surface.configure(&app.device, &app.surface_conf);
			app.enable_depth_test();
			self.is_need_resize = false;
		}
	}
	pub fn set_window_resized(&mut self, n_size: PhysicalSize<u32>) {
		if n_size == self.current_size {
			return;
		}
		self.current_size = n_size;
		self.is_need_resize = true;
	}
}
