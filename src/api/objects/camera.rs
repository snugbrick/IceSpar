use glam::{Mat4, Vec3};
use russimp_ng::camera::Camera;
use wgpu::{
	BindGroupEntry, BindGroupLayoutEntry, Buffer, BufferUsages, Device, Queue, ShaderStages,
};

use crate::{
	ingredient::{bindgroup::BindGroupIS, buffer::BufferIS},
	utilities::russimp2glam,
};

pub struct CameraIS {
	pub camera: Camera, // using its #[test] :)
	pub yaw: f32,
	pub pitch: f32,
	pub distance: f32,
	pub view_proj_buffer: Option<Buffer>,
	pub cam_pos_buffer: Option<Buffer>,
	pub bindgroup: Option<BindGroupIS>,
}

impl CameraIS {
	pub fn from_russimp_cam(cam: Camera) -> Self {
		let pos = Vec3::new(cam.position.x, cam.position.y, cam.position.z);
		let look_at = Vec3::new(cam.look_at.x, cam.look_at.y, cam.look_at.z);
		let delta = pos - look_at;
		let distance = delta.length();
		let yaw = delta.z.atan2(delta.x);
		let pitch = (delta.y / distance).asin();
		CameraIS {
			camera: cam,
			yaw,
			pitch,
			distance,
			view_proj_buffer: None,
			cam_pos_buffer: None,
			bindgroup: None,
		}
	}

	pub fn cal_view_proj_mat(&self) -> Mat4 {
		let cam_position = russimp2glam::vec3to_glam(&self.camera.position);
		let lookat = russimp2glam::vec3to_glam(&self.camera.look_at);
		let up = russimp2glam::vec3to_glam(&self.camera.up);

		let view_mat = Mat4::look_at_rh(cam_position, lookat, up);
		let proj_mat = Mat4::perspective_rh(
			self.camera.horizontal_fov,
			self.camera.aspect,
			self.camera.clip_plane_near,
			self.camera.clip_plane_far,
		);
		proj_mat * view_mat
	}

	pub fn create_gpu_resources(&mut self, device: &Device) {
		let initial_mat = self.cal_view_proj_mat();
		let view_proj_buffer = BufferIS::new(
			device,
			bytemuck::cast_slice(&initial_mat.to_cols_array_2d()),
			BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		)
		.buffer;

		let initial_pos: [f32; 4] = [
			self.camera.position.x,
			self.camera.position.y,
			self.camera.position.z,
			1.0,
		];
		let cam_pos_buffer = BufferIS::new(
			device,
			bytemuck::cast_slice(&initial_pos),
			BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		)
		.buffer;

		let bindgroup = BindGroupIS::new(
			device,
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

		self.view_proj_buffer = Some(view_proj_buffer);
		self.cam_pos_buffer = Some(cam_pos_buffer);
		self.bindgroup = Some(bindgroup);
	}

	pub fn write_gpu_buffers(&self, queue: &Queue) {
		let view_proj = self.cal_view_proj_mat();
		queue.write_buffer(
			self.view_proj_buffer.as_ref().unwrap(),
			0,
			bytemuck::cast_slice(&view_proj.to_cols_array_2d()),
		);
		let cam_pos: [f32; 4] = [
			self.camera.position.x,
			self.camera.position.y,
			self.camera.position.z,
			1.0,
		];
		queue.write_buffer(
			self.cam_pos_buffer.as_ref().unwrap(),
			0,
			bytemuck::cast_slice(&cam_pos),
		);
	}

	pub fn apply_orbit_rotation(
		&mut self,
		mouse_delta: (f64, f64),
		sensitivity: f32,
		pitch_clamp: (f32, f32),
	) {
		self.yaw += mouse_delta.0 as f32 * sensitivity;
		self.pitch += mouse_delta.1 as f32 * sensitivity;
		self.pitch = self.pitch.clamp(pitch_clamp.0, pitch_clamp.1);
	}

	pub fn apply_zoom(&mut self, scroll: (f64, f64), sensitivity: f32, distance_clamp: (f32, f32)) {
		self.distance -= scroll.1 as f32 * sensitivity;
		self.distance = self.distance.clamp(distance_clamp.0, distance_clamp.1);
	}

	pub fn update_from_orbit(&mut self) {
		self.camera.position.x =
			self.camera.look_at.x + self.distance * self.pitch.cos() * self.yaw.cos();
		self.camera.position.y = self.camera.look_at.y + self.distance * self.pitch.sin();
		self.camera.position.z =
			self.camera.look_at.z + self.distance * self.pitch.cos() * self.yaw.sin();
	}

	pub fn forward(&self) -> Vec3 {
		Vec3::new(
			self.pitch.cos() * self.yaw.cos(),
			self.pitch.sin(),
			self.pitch.cos() * self.yaw.sin(),
		)
	}

	pub fn right(&self) -> Vec3 {
		Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos()).normalize()
	}

	pub fn pan_forward(&mut self, amount: f32) {
		let fwd = self.forward();
		self.camera.look_at.x -= fwd.x * amount;
		self.camera.look_at.y -= fwd.y * amount;
		self.camera.look_at.z -= fwd.z * amount;
	}

	pub fn pan_right(&mut self, amount: f32) {
		let r = self.right();
		self.camera.look_at.x += r.x * amount;
		self.camera.look_at.y += r.y * amount;
		self.camera.look_at.z += r.z * amount;
	}
}
