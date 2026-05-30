#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex_IS {
	pub position: [f32; 3],
	pub uv: [f32; 2],
	pub normal: [f32; 3],
	pub tangent: [f32; 3],
	pub bitangent: [f32; 3],
}
