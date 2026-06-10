use std::borrow::Cow;

use russimp_ng::camera::Camera;
use wgpu::{ColorTargetState, Device, FragmentState, SurfaceConfiguration, VertexState};

use crate::{
	api::{
		init_conf::{CameraConfig, InputConfig},
		objects::{camera::CameraIS, object::ObjectIS},
	},
	app::event_handle::EventHandle,
	ingredient::{
		bindgroup::BindGroupIS, buffer::VIBufferFromMesh, buffer_layout::vertex_buffer_layout_vi,
		pipeline::PipelineIS, texture::TextureIS,
	},
	resources::model::Model,
};

pub struct SceneIS {
	pub objects: Vec<ObjectIS>,
	pub cameras: Vec<CameraIS>,
	pub is_render_objects: bool,
	pub mesh_buffers: Vec<VIBufferFromMesh<'static>>,
	pub material_bindgroups: Vec<BindGroupIS>,
	pub pipeline: Option<PipelineIS>,
}

pub struct SceneRenderData<'a> {
	pub pipeline: &'a PipelineIS,
	pub mesh_buffers: &'a [VIBufferFromMesh<'static>],
	pub camera_bindgroup: &'a BindGroupIS,
	pub material_bindgroups: &'a [BindGroupIS],
}

impl SceneIS {
	pub fn add_camera(&mut self, config: &CameraConfig, aspect: f32, device: &Device) {
		let mut camera = CameraIS::from_russimp_cam(Camera {
			name: String::from("main camera"),
			aspect,
			clip_plane_far: config.far,
			clip_plane_near: config.near,
			horizontal_fov: config.fov,
			look_at: russimp_ng::Vector3D {
				x: config.look_at[0],
				y: config.look_at[1],
				z: config.look_at[2],
			},
			position: russimp_ng::Vector3D {
				x: config.position[0],
				y: config.position[1],
				z: config.position[2],
			},
			up: russimp_ng::Vector3D {
				x: config.up[0],
				y: config.up[1],
				z: config.up[2],
			},
		});
		camera.create_gpu_resources(device);
		self.cameras.push(camera);
	}

	pub fn add_model(&mut self, path: &str, device: &Device, queue: &wgpu::Queue) {
		let model = Model::new(String::from(path)).init_nodes_tree();

		self.mesh_buffers = model
			.content
			.meshes
			.iter()
			.map(|mesh| VIBufferFromMesh::new(device, mesh))
			.collect();

		let default_texs = TextureIS::create_default_textures(device, queue);
		let texlors = model.texlors.as_ref().unwrap();
		self.material_bindgroups = texlors
			.iter()
			.map(|tex_unit| TextureIS::create_material_bindgroup(device, queue, tex_unit, &default_texs))
			.collect();
	}

	pub fn build_pipeline(&mut self, device: &Device, surface_conf: &SurfaceConfiguration) {
		if self.cameras.is_empty() || self.material_bindgroups.is_empty() {
			return;
		}

		let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/shader.wgsl"))),
		});

		self.pipeline = Some(
			PipelineIS::begin_layout(
				device,
				&[
					&self.cameras[0].bindgroup.as_ref().unwrap().bindgrouplayout,
					&self.material_bindgroups[0].bindgrouplayout,
				],
			)
			.set_depthstencil_state(wgpu::DepthStencilState {
				format: wgpu::TextureFormat::Depth32Float,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default(),
			})
			.get_pipeline(
				device,
				VertexState {
					module: &shader_module,
					entry_point: "vs_main",
					buffers: &[vertex_buffer_layout_vi().buffer_layout],
				},
				FragmentState {
					module: &shader_module,
					entry_point: "fs_main",
					targets: &[Some(ColorTargetState {
						format: surface_conf.format,
						blend: Some(wgpu::BlendState::REPLACE),
						write_mask: wgpu::ColorWrites::ALL,
					})],
				},
			),
		);
	}

	pub fn update_camera(
		&mut self,
		camera_index: usize,
		eh: &EventHandle,
		dt: f32,
		input_cfg: &InputConfig,
		queue: &wgpu::Queue,
	) {
		let Some(camera) = self.cameras.get_mut(camera_index) else {
			return;
		};
		let speed = input_cfg.pan_speed * dt;

		if eh.is_mouse_pressed(input_cfg.orbit_button) {
			camera.apply_orbit_rotation(
				eh.cursor_delta,
				input_cfg.orbit_sensitivity,
				(input_cfg.pitch_clamp_min, input_cfg.pitch_clamp_max),
			);
		}
		camera.apply_zoom(
			eh.scroll_delta,
			input_cfg.zoom_sensitivity,
			(input_cfg.distance_clamp_min, input_cfg.distance_clamp_max),
		);

		if eh.is_key_pressed(input_cfg.forward_key) {
			camera.pan_forward(speed);
		}
		if eh.is_key_pressed(input_cfg.backward_key) {
			camera.pan_forward(-speed);
		}
		if eh.is_key_pressed(input_cfg.left_key) {
			camera.pan_right(-speed);
		}
		if eh.is_key_pressed(input_cfg.right_key) {
			camera.pan_right(speed);
		}

		camera.update_from_orbit();
		camera.write_gpu_buffers(queue);
	}

	pub fn render_data(&self) -> Option<SceneRenderData<'_>> {
		Some(SceneRenderData {
			pipeline: self.pipeline.as_ref()?,
			mesh_buffers: &self.mesh_buffers,
			camera_bindgroup: self.cameras.first()?.bindgroup.as_ref()?,
			material_bindgroups: &self.material_bindgroups,
		})
	}
}
