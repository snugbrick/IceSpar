use wgpu::Color;
use winit::event_loop::EventLoop;

use crate::{
	api::{init_conf::InitConfig, scene_is::SceneIS},
	app::{app::App, event_handle::EventHandle, render::RenderPreparation},
};

pub struct InstanceIS {
	pub the_scene: Vec<SceneIS>,
	pub app: App,
	pub event_loop: EventLoop<()>,
	pub config: InitConfig,
}

impl InstanceIS {
	pub fn get_instance() -> Self {
		Self::get(InitConfig::default())
	}

	pub fn get(config: InitConfig) -> Self {
		let event_loop = EventLoop::new().unwrap();
		let mut app = App::instance(&event_loop, config.window.title, config.render.present_mode);
		app.enable_depth_test(config.render.depth_format);

		let aspect = app.surface_conf.width as f32 / app.surface_conf.height as f32;
		let mut scene = SceneIS {
			objects: Vec::new(),
			cameras: Vec::new(),
			is_render_objects: true,
			mesh_buffers: Vec::new(),
			material_bindgroups: Vec::new(),
			pipeline: None,
		};
		scene.add_camera(&config.camera, aspect, &app.device);

		InstanceIS {
			the_scene: vec![scene],
			app,
			event_loop,
			config,
		}
	}

	pub fn run(mut self, scene_index: usize) {
		let event_loop = self.event_loop;
		let input_cfg = self.config.input;
		let render_cfg = self.config.render;
		self.the_scene[scene_index].build_pipeline(&self.app.device, &self.app.surface_conf);

		let app = self.app;

		let clear_color = Color {
			r: render_cfg.clear_color[0],
			g: render_cfg.clear_color[1],
			b: render_cfg.clear_color[2],
			a: render_cfg.clear_color[3],
		};

		let event_handle = EventHandle::new();

		EventHandle::run(event_handle, event_loop, app, move |eh, app| {
			let dt = eh.delta_time as f32;
			self.the_scene[scene_index].update_camera(0, eh, dt, &input_cfg, &app.queue);

			if let Some(rd) = self.the_scene[scene_index].render_data() {
				let mut rp = RenderPreparation::prepare_render(app);
				let material_bg_list: Vec<&wgpu::BindGroup> = rd
					.material_bindgroups
					.iter()
					.map(|bg| &bg.bindgroup)
					.collect();
				rp.record_renderpass(
					rd.pipeline,
					rd.mesh_buffers,
					&[&rd.camera_bindgroup.bindgroup],
					Some(&material_bg_list),
					clear_color,
				);
				rp.present(&app.queue);
			}
		});
	}
}
