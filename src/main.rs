use std::borrow::Cow;

use ice_spar::{
	app::{app::App, event_handle::EventHandle, render::RenderPreparation},
	ingredient::{
		bindgroup::BindGroupIS,
		buffer::{BufferIS, VIBufferFromMesh},
		buffer_layout::vertex_buffer_layout_vi,
		objects::camera::CameraIS,
		pipeline::PipelineIS,
		texture::TextureIS,
	},
	resources::model::Model,
};
use russimp_ng::{Vector3D, camera::Camera};
use wgpu::{
	BindGroupEntry, BindGroupLayoutEntry, BufferUsages, ColorTargetState, FragmentState,
	ShaderStages, VertexState,
};
use winit::{event::MouseButton, event_loop::EventLoop, keyboard::KeyCode};

fn main() {
	let event_loop = EventLoop::new().unwrap();
	let mut app = App::instance(&event_loop);
	app.enable_depth_test();

	let model = Model::new(String::from("src/blueberry.fbx")).init_nodes_tree();

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

	let initial_mat = camera.cal_view_proj_mat();
	let view_proj_buffer = BufferIS::new(
		&app.device,
		bytemuck::cast_slice(&initial_mat.to_cols_array_2d()),
		BufferUsages::UNIFORM | BufferUsages::COPY_DST,
	)
	.buffer;

	let initial_pos: [f32; 4] = [
		camera.camera.position.x,
		camera.camera.position.y,
		camera.camera.position.z,
		1.0,
	];
	let cam_pos_buffer = BufferIS::new(
		&app.device,
		bytemuck::cast_slice(&initial_pos),
		BufferUsages::UNIFORM | BufferUsages::COPY_DST,
	)
	.buffer;

	let camera_bindgroup = BindGroupIS::new(
		&app.device,
		&[
			BindGroupLayoutEntry {
				binding: 0,
				visibility: ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			BindGroupLayoutEntry {
				binding: 1,
				visibility: ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
		],
		&[
			BindGroupEntry {
				binding: 0,
				resource: view_proj_buffer.as_entire_binding(),
			},
			BindGroupEntry {
				binding: 1,
				resource: cam_pos_buffer.as_entire_binding(),
			},
		],
	);

	let default_texs = TextureIS::create_default_textures(&app.device, &app.queue);

	let texlors = model.texlors.as_ref().unwrap();
	let mut material_bindgroups: Vec<BindGroupIS> = Vec::new();

	for tex_unit in texlors {
		let bindgroup = TextureIS::create_material_bindgroup(
			&app.device,
			&app.queue,
			tex_unit,
			&default_texs,
		);
		material_bindgroups.push(bindgroup);
	}

	let shader_module = app
		.device
		.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../src/shaders/shader.wgsl"))),
		});

	let pipeline = PipelineIS::begin_layout(
		&app.device,
		&[
			&camera_bindgroup.bindgrouplayout,
			&material_bindgroups[0].bindgrouplayout,
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
		&app.device,
		VertexState {
			module: &shader_module,
			entry_point: "vs_main",
			buffers: &[vertex_buffer_layout_vi().buffer_layout],
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

	let look_at = glam::Vec3::new(
		camera.camera.look_at.x,
		camera.camera.look_at.y,
		camera.camera.look_at.z,
	);
	let pos = glam::Vec3::new(
		camera.camera.position.x,
		camera.camera.position.y,
		camera.camera.position.z,
	);
	let delta = pos - look_at;
	let mut distance = delta.length();
	let mut yaw = delta.z.atan2(delta.x);
	let mut pitch = (delta.y / distance).asin();

	let event_handle = EventHandle::new();

	EventHandle::run(event_handle, event_loop, app, move |eh, app| {
		let dt = eh.delta_time as f32;
		let speed = 300.0 * dt;
		let sensitivity = 0.005f32;

		if eh.is_mouse_pressed(MouseButton::Right) {
			yaw += eh.cursor_delta.0 as f32 * sensitivity;
			pitch += eh.cursor_delta.1 as f32 * sensitivity;
			pitch = pitch.clamp(-1.5, 1.5);
		}
		distance -= eh.scroll_delta.1 as f32 * 3.0;
		distance = distance.clamp(1.0, 10000.0);

		let forward = glam::Vec3::new(
			pitch.cos() * yaw.cos(),
			pitch.sin(),
			pitch.cos() * yaw.sin(),
		);
		let right = glam::Vec3::new(yaw.sin(), 0.0, -yaw.cos()).normalize();

		if eh.is_key_pressed(KeyCode::KeyS) {
			camera.camera.look_at.x += forward.x * speed;
			camera.camera.look_at.y += forward.y * speed;
			camera.camera.look_at.z += forward.z * speed;
		}
		if eh.is_key_pressed(KeyCode::KeyW) {
			camera.camera.look_at.x -= forward.x * speed;
			camera.camera.look_at.y -= forward.y * speed;
			camera.camera.look_at.z -= forward.z * speed;
		}
		if eh.is_key_pressed(KeyCode::KeyA) {
			camera.camera.look_at.x -= right.x * speed;
			camera.camera.look_at.y -= right.y * speed;
			camera.camera.look_at.z -= right.z * speed;
		}
		if eh.is_key_pressed(KeyCode::KeyD) {
			camera.camera.look_at.x += right.x * speed;
			camera.camera.look_at.y += right.y * speed;
			camera.camera.look_at.z += right.z * speed;
		}

		camera.camera.position.x = camera.camera.look_at.x + distance * pitch.cos() * yaw.cos();
		camera.camera.position.y = camera.camera.look_at.y + distance * pitch.sin();
		camera.camera.position.z = camera.camera.look_at.z + distance * pitch.cos() * yaw.sin();
		let mat_new = camera.cal_view_proj_mat();
		app.queue.write_buffer(
			&view_proj_buffer,
			0,
			bytemuck::cast_slice(&mat_new.to_cols_array_2d()),
		);
		let cam_pos: [f32; 4] = [
			camera.camera.position.x,
			camera.camera.position.y,
			camera.camera.position.z,
			1.0,
		];
		app.queue.write_buffer(&cam_pos_buffer, 0, bytemuck::cast_slice(&cam_pos));

		let mut rp = RenderPreparation::prepare_render(app);
		let material_bg_list: Vec<&wgpu::BindGroup> =
			material_bindgroups.iter().map(|bg| &bg.bindgroup).collect();
		rp.record_renderpass(
			&pipeline,
			&mesh_buffers,
			&[&camera_bindgroup.bindgroup],
			Some(&material_bg_list),
		);
		rp.present(&app.queue);
	});
}
