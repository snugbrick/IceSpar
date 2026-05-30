use glam::Mat4;
use russimp_ng::camera::Camera;

use crate::utilities::russimp2glam;

pub struct CameraIS {
	pub camera: Camera,
}

impl CameraIS {
	pub fn from_russimp_cam(cam: Camera) -> Self {
		CameraIS { camera: cam }
	}
	pub fn cal_view_proj_mat(&self) -> Mat4 {
		let cam_position = russimp2glam::vec3to_glam(&self.camera.position);
		let lookat = russimp2glam::vec3to_glam(&self.camera.look_at);
		let up = russimp2glam::vec3to_glam(&self.camera.up);

		let view_mat = glam::Mat4::look_at_rh(cam_position, lookat, up);
		let proj_mat = glam::Mat4::perspective_rh(
			self.camera.horizontal_fov,
			self.camera.aspect,
			self.camera.clip_plane_near,
			self.camera.clip_plane_far,
		);
		proj_mat * view_mat
	}
}
