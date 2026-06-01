use std::collections::HashSet;
use std::time::Instant;

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

use crate::app::app::App;

pub struct EventHandle {
	keys_pressed: HashSet<KeyCode>,
	keys_prev: HashSet<KeyCode>,
	modifiers: ModifiersState,

	mouse_buttons_pressed: HashSet<MouseButton>,
	mouse_buttons_prev: HashSet<MouseButton>,
	pub cursor_position: PhysicalPosition<f64>,
	pub cursor_delta: (f64, f64),
	pub scroll_delta: (f64, f64),

	pub close_requested: bool,
	pub window_resize: Option<PhysicalSize<u32>>,

	pub delta_time: f64,
	last_frame: Instant,
}

impl EventHandle {
	pub fn new() -> Self {
		Self {
			keys_pressed: HashSet::new(),
			keys_prev: HashSet::new(),
			modifiers: ModifiersState::empty(),

			mouse_buttons_pressed: HashSet::new(),
			mouse_buttons_prev: HashSet::new(),
			cursor_position: PhysicalPosition::new(0.0, 0.0),
			cursor_delta: (0.0, 0.0),
			scroll_delta: (0.0, 0.0),

			close_requested: false,
			window_resize: None,

			delta_time: 0.0,
			last_frame: Instant::now(),
		}
	}

	pub fn process_event(&mut self, event: &Event<()>) {
		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => {
					self.close_requested = true;
				}
				WindowEvent::Resized(size) => {
					self.window_resize = Some(*size);
				}
				WindowEvent::KeyboardInput {
					event: key_event, ..
				} => {
					if let PhysicalKey::Code(keycode) = key_event.physical_key {
						match key_event.state {
							ElementState::Pressed => {
								self.keys_pressed.insert(keycode);
							}
							ElementState::Released => {
								self.keys_pressed.remove(&keycode);
							}
						}
					}
				}
				WindowEvent::MouseInput { state, button, .. } => match state {
					ElementState::Pressed => {
						self.mouse_buttons_pressed.insert(*button);
					}
					ElementState::Released => {
						self.mouse_buttons_pressed.remove(&button);
					}
				},
				WindowEvent::CursorMoved { position, .. } => {
					self.cursor_delta.0 += position.x - self.cursor_position.x;
					self.cursor_delta.1 += position.y - self.cursor_position.y;
					self.cursor_position = *position;
				}
				WindowEvent::MouseWheel { delta, .. } => match delta {
					MouseScrollDelta::LineDelta(x, y) => {
						self.scroll_delta = (*x as f64, *y as f64);
					}
					MouseScrollDelta::PixelDelta(pos) => {
						self.scroll_delta = (pos.x, pos.y);
					}
				},
				WindowEvent::ModifiersChanged(modifiers) => {
					self.modifiers = modifiers.state();
				}
				_ => {}
			},
			_ => {}
		}
	}

	pub fn end_frame(&mut self) {
		self.keys_prev = self.keys_pressed.clone();
		self.mouse_buttons_prev = self.mouse_buttons_pressed.clone();
		self.scroll_delta = (0.0, 0.0);
		self.cursor_delta = (0.0, 0.0);
	}

	pub fn is_key_pressed(&self, key: KeyCode) -> bool {
		self.keys_pressed.contains(&key)
	}

	pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
		self.keys_pressed.contains(&key) && !self.keys_prev.contains(&key)
	}

	pub fn is_key_just_released(&self, key: KeyCode) -> bool {
		!self.keys_pressed.contains(&key) && self.keys_prev.contains(&key)
	}

	pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
		self.mouse_buttons_pressed.contains(&button)
	}

	pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
		self.mouse_buttons_pressed.contains(&button) && !self.mouse_buttons_prev.contains(&button)
	}

	pub fn is_mouse_just_released(&self, button: MouseButton) -> bool {
		!self.mouse_buttons_pressed.contains(&button) && self.mouse_buttons_prev.contains(&button)
	}

	pub fn is_shift(&self) -> bool {
		self.modifiers.shift_key()
	}

	pub fn is_ctrl(&self) -> bool {
		self.modifiers.control_key()
	}

	pub fn is_alt(&self) -> bool {
		self.modifiers.alt_key()
	}

	pub fn is_super(&self) -> bool {
		self.modifiers.super_key()
	}

	pub fn is_modifier(&self, modifier: ModifiersState) -> bool {
		self.modifiers.contains(modifier)
	}

	pub fn run<F>(mut self, event_loop: EventLoop<()>, mut app: App, mut callback: F)
	where
		F: FnMut(&mut Self, &mut App) + 'static,
	{
		let _ = event_loop.run(move |event, elwt| {
			elwt.set_control_flow(ControlFlow::Poll);
			self.process_event(&event);

			if self.close_requested {
				elwt.exit();
				return;
			}

			if matches!(&event, Event::AboutToWait) {
				let now = Instant::now();
				self.delta_time = (now - self.last_frame).as_secs_f64();
				self.last_frame = now;

				if let Some(size) = self.window_resize.take() {
					app.surface_conf.width = size.width;
					app.surface_conf.height = size.height;
					app.surface.configure(&app.device, &app.surface_conf);
					app.enable_depth_test();
				}
				callback(&mut self, &mut app);
				self.end_frame();
			}
		});
	}
}
