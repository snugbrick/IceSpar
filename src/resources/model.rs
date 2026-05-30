use std::rc::Rc;

use glam::Mat4;
use russimp_ng::{
	animation::Animation,
	camera::Camera,
	light::Light,
	material::Material,
	mesh::Mesh,
	node::Node,
	scene::{PostProcess, Scene},
};

use crate::utilities::russimp2glam;

pub struct Model {
	pub path: String,
	pub content: ModelContent,
}

pub struct ModelContent {
	pub root_node: Rc<Node>,
	pub meshes: Vec<Mesh>,
	pub material: Vec<Material>,
	pub lights: Vec<Light>,
	pub camera: Vec<Camera>,
	pub animation: Vec<Animation>,
	pub flags: u32,
}

impl Model {
	pub fn new(path: String) -> Self {
		if path.is_empty() {
			panic!("path cant be empty")
		}
		let scene = Scene::from_file(
			&path,
			vec![
				PostProcess::Triangulate,
				PostProcess::CalculateTangentSpace,
				PostProcess::JoinIdenticalVertices,
				PostProcess::SortByPrimitiveType,
				PostProcess::GenerateNormals,
				PostProcess::GenerateUVCoords,
				PostProcess::OptimizeMeshes,
				PostProcess::FixInfacingNormals,
				PostProcess::FindDegenerates,
				PostProcess::ImproveCacheLocality,
			],
		)
		.unwrap();

		let _metadata = scene.metadata.unwrap();

		Self {
			path: path,
			content: ModelContent {
				root_node: scene.root.unwrap(),
				meshes: scene.meshes,
				material: scene.materials,
				lights: scene.lights,
				camera: scene.cameras,
				animation: scene.animations,
				flags: scene.flags,
			},
		}
	}

	pub fn transform_to_world_space(mut self) -> Self {
		let root_node = &self.content.root_node;
		let root_matrix = russimp2glam::mat4to_glam(&root_node.transformation);

		Self::transform_nodes(root_node, root_matrix, &mut self.content.meshes);
		self
	}

	fn transform_nodes(node: &Node, matrix: Mat4, meshes: &mut Vec<Mesh>) {
		for &mesh_index in &node.meshes {
			let mesh = &mut meshes[mesh_index as usize];
			for vertex in mesh.vertices.iter_mut() {
				let v = glam::Vec3::new(vertex.x, vertex.y, vertex.z);
				let transformed = matrix.transform_point3(v);
				vertex.x = transformed.x;
				vertex.y = transformed.y;
				vertex.z = transformed.z;
			}
		}

		for child in node.children.borrow().iter() {
			let child_matrix = matrix * russimp2glam::mat4to_glam(&child.transformation);
			Self::transform_nodes(child, child_matrix, meshes);
		}
	}
}
