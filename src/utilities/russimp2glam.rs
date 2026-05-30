use glam::{Mat4, Vec3};
use russimp_ng::{Matrix4x4, Vector3D};

pub fn mat4to_glam(mat: &Matrix4x4) -> Mat4 {
	Mat4::from_cols_array(&[
		mat.a1, mat.b1, mat.c1, mat.d1, mat.a2, mat.b2, mat.c2, mat.d2, mat.a3, mat.b3, mat.c3, mat.d3,
		mat.a4, mat.b4, mat.c4, mat.d4,
	])
}
pub fn vec3to_glam(vec3: &Vector3D) -> Vec3 {
	Vec3::from_array([vec3.x, vec3.y, vec3.z])
}
