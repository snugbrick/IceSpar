use std::borrow::Cow;

use glam::{Mat4, Vec3};
use ice_spar::{
	app::{app::App, event_handle::EventHandle, render::RenderPreparation},
	ingredient::{
		bindgroup::BindGroupIS, buffer::VIBufferFromMesh, buffer_layout::vertex_buffer_layout_vi,
		objects::camera::CameraIS, pipeline::PipelineIS,
	},
	resources::model::Model,
};
use russimp_ng::{Vector3D, camera::Camera};
use wgpu::{
	BindGroupEntry, BindGroupLayoutEntry, BufferUsages, ColorTargetState, FragmentState,
	ShaderStages, VertexState, util::DeviceExt,
};
use winit::{event_loop::EventLoop, keyboard::KeyCode};

fn main() {
	let event_loop = EventLoop::new().unwrap();
	let mut app = App::instance(&event_loop);
	app.enable_depth_test();

	let model = Model::new(String::from("../src/tree.fbx")).transform_to_world_space();

	let mesh_buffers: Vec<VIBufferFromMesh> = model
		.content
		.meshes
		.iter()
		.map(|mesh| VIBufferFromMesh::new(&app.device, mesh))
		.collect();

	let mut camera = CameraIS {
		camera: Camera {
			name: String::from("main camera"),
			aspect: app.surface_conf.width as f32 / app.surface_conf.height as f32,
			clip_plane_far: 10000.0,
			clip_plane_near: 1.0,
			horizontal_fov: 60.0f32.to_radians(),
			look_at: Vector3D {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			position: Vector3D {
				x: 800.0,
				y: 800.0,
				z: 800.0,
			},
			up: Vector3D {
				x: 0.0,
				y: 1.0,
				z: 0.0,
			},
		},
	};

	let uniform_buffer = app
		.device
		.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Uniform Buffer"),
			contents: bytemuck::cast_slice(&camera.cal_view_proj_mat().to_cols_array_2d()),
			usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		});

	let camera_bindgroup = BindGroupIS::new(
		&app.device,
		&[BindGroupLayoutEntry {
			binding: 0,
			visibility: ShaderStages::VERTEX,
			ty: wgpu::BindingType::Buffer {
				ty: wgpu::BufferBindingType::Uniform,
				has_dynamic_offset: false,
				min_binding_size: None,
			},
			count: None,
		}],
		&[BindGroupEntry {
			binding: 0,
			resource: uniform_buffer.as_entire_binding(),
		}],
	);

	let shader_module = app
		.device
		.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/shader.wgsl"))),
		});

	let layout = vertex_buffer_layout_vi();
	let pipeline = PipelineIS::begin_layout(&app.device, &[&camera_bindgroup.bindgrouplayout])
		.set_depthstencil_state(wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth32Float,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		})
		.get_pipeline(
			&app.device,
			VertexState {
				module: &shader_module,
				entry_point: "vs_main",
				buffers: &[layout.buffer_layout],
			},
			FragmentState {
				module: &shader_module,
				entry_point: "fs_main",
				targets: &[Some(ColorTargetState {
					format: app.surface_conf.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			},
		);

	let event_handle = EventHandle::new();

	EventHandle::run(event_handle, event_loop, app, move |eh, app| {
		let dt = eh.delta_time as f32;

		if eh.is_key_pressed(KeyCode::KeyW) {
			camera.camera.position.z -= 100.0 * dt;
		}
		if eh.is_key_pressed(KeyCode::KeyS) {
			camera.camera.position.z += 100.0 * dt;
		}
		if eh.is_key_pressed(KeyCode::KeyA) {
			camera.camera.position.x -= 100.0 * dt;
		}
		if eh.is_key_pressed(KeyCode::KeyD) {
			camera.camera.position.x += 100.0 * dt;
		}
		let mat_new = camera.cal_view_proj_mat();
		app.queue.write_buffer(
			&uniform_buffer,
			0,
			bytemuck::cast_slice(&mat_new.to_cols_array_2d()),
		);

		let mut rp = RenderPreparation::prepare_render(app);
		rp.record_renderpass(&pipeline, &mesh_buffers, &[&camera_bindgroup.bindgroup]);
		rp.present(&app.queue);
	});
}
