use std::mem;

use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat};

pub struct BufferLayoutIS<'a> {
	pub buffer_layout: VertexBufferLayout<'a>,
}
impl<'a> BufferLayoutIS<'a> {
	pub fn new(vertex_attr: &'a [VertexAttribute], len: u64) -> Self {
		Self {
			buffer_layout: VertexBufferLayout {
				array_stride: len,
				step_mode: wgpu::VertexStepMode::Vertex,
				attributes: vertex_attr,
			},
		}
	}
}

pub fn vertex_buffer_layout_vi<'a>() -> BufferLayoutIS<'a> {
	let vi_buffer_layout = BufferLayoutIS::new(
		&[
			// vertices
			VertexAttribute {
				format: VertexFormat::Float32x3,
				offset: 0,
				shader_location: 0,
			},
			// uv
			VertexAttribute {
				format: VertexFormat::Float32x2,
				offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
				shader_location: 1,
			},
			// normal
			VertexAttribute {
				format: VertexFormat::Float32x3,
				offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
				shader_location: 2,
			},
			// tangent
			VertexAttribute {
				format: VertexFormat::Float32x3,
				offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
				shader_location: 3,
			},
			// bitangent
			VertexAttribute {
				format: VertexFormat::Float32x3,
				offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
				shader_location: 4,
			},
		],
		56 as u64,
	);
	vi_buffer_layout
}
