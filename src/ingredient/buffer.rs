use russimp_ng::mesh::Mesh;
use wgpu::{Buffer, BufferUsages, Device, VertexBufferLayout, util::DeviceExt};

use crate::{ingredient::buffer_layout::vertex_buffer_layout_vi, utilities::vertex::Vertex_IS};

pub struct BufferIS {
	pub buffer: Buffer,
}
impl BufferIS {
	pub fn new(device: &Device, content: &[u8], usage: BufferUsages) -> Self {
		Self {
			buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("buffer"),
				contents: content,
				usage: usage,
			}),
		}
	}
}

pub struct VertexBufferIS<'a> {
	pub buffer: Buffer,
	pub buffer_layout: VertexBufferLayout<'a>,
}
pub trait VertexBufferIS4BufferIS<'a> {
	fn create_vertex_buffer(device: &Device, content: &Vec<Vertex_IS>) -> VertexBufferIS<'a>;
}
impl<'a> VertexBufferIS4BufferIS<'a> for BufferIS {
	fn create_vertex_buffer(device: &Device, content: &Vec<Vertex_IS>) -> VertexBufferIS<'a> {
		VertexBufferIS {
			buffer: BufferIS::new(device, bytemuck::cast_slice(content), BufferUsages::VERTEX).buffer,
			buffer_layout: vertex_buffer_layout_vi().buffer_layout,
		}
	}
}

pub trait IndexBufferIS {
	fn create_index_buffer(device: &Device, content: &Vec<u32>) -> Self;
}
impl IndexBufferIS for BufferIS {
	fn create_index_buffer(device: &Device, content: &Vec<u32>) -> Self {
		BufferIS::new(device, bytemuck::cast_slice(content), BufferUsages::INDEX)
	}
}

pub struct VIBufferFromMesh<'a> {
	pub vertex_buffer: VertexBufferIS<'a>,
	pub index_buffer: Buffer,
	pub vertex_count: u32,
	pub index_count: u32,
}

impl<'a> VIBufferFromMesh<'a> {
	pub fn new(device: &Device, meshes: &Mesh) -> Self {
		let indices: Vec<u32> = meshes
			.faces
			.iter()
			.flat_map(|face| face.0.iter().copied())
			.collect();

		let vertices = &meshes.vertices;
		let uv = meshes
			.texture_coords
			.first()
			.and_then(|opt| opt.as_ref())
			.unwrap();
		let normals = &meshes.normals;
		let tangents = &meshes.tangents;
		let bitangents = &meshes.bitangents;

		let vertex_vec: Vec<Vertex_IS> = vertices
			.iter()
			.enumerate()
			.map(|(i, pos)| Vertex_IS {
				position: [pos.x, pos.y, pos.z],
				uv: uv.get(i).map_or([0.0, 0.0], |v| [v.x, v.y]),
				normal: normals.get(i).map_or([0.0, 0.0, 0.0], |n| [n.x, n.y, n.z]),
				tangent: tangents.get(i).map_or([0.0, 0.0, 0.0], |t| [t.x, t.y, t.z]),
				bitangent: bitangents
					.get(i)
					.map_or([0.0, 0.0, 0.0], |b| [b.x, b.y, b.z]),
			})
			.collect();

		let vertex_count = vertices.len();
		let index_count = indices.len();

		Self {
			vertex_buffer: BufferIS::create_vertex_buffer(&device, &vertex_vec),
			index_buffer: BufferIS::create_index_buffer(&device, &indices).buffer,
			vertex_count: vertex_count as u32,
			index_count: index_count as u32,
		}
	}
}
