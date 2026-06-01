use std::{cell::RefCell, collections::HashMap, rc::Rc};

use glam::Vec4;
use russimp_ng::{
	material::{Material, PropertyTypeInfo, Texture, TextureType},
	mesh::Mesh,
};

pub struct ISTextureUnit {
	pub texture: Option<HashMap<usize, Rc<RefCell<Texture>>>>,
	pub diffuse_color: Option<HashMap<usize, Vec4>>,
}

impl ISTextureUnit {
	pub fn from_mesh(
		mesh: &Vec<Mesh>,
		mesh_index: usize,
		materials: &Vec<Material>,
		texture_type: TextureType,
	) -> Self {
		let c_mesh = &mesh[mesh_index];
		let the_material = &materials[c_mesh.material_index as usize];

		let mut textures = HashMap::new();
		if let Some(the_texture) = the_material.textures.get(&texture_type) {
			textures.insert(mesh_index, the_texture.clone());
		}

		let mut diffuse_color = HashMap::new();
		diffuse_color.insert(mesh_index, Self::get_diffuse_color(the_material));
		Self {
			texture: if textures.is_empty() {
				None
			} else {
				Some(textures)
			},
			diffuse_color: Some(diffuse_color),
		}
	}
	fn get_diffuse_color(material: &Material) -> Vec4 {
		let mut rgba = Vec4::new(1.0, 1.0, 1.0, 1.0);
		for prop in &material.properties {
			if prop.key == "$clr.diffuse" {
				if let PropertyTypeInfo::FloatArray(vec) = &prop.data {
					if vec.len() >= 3 {
						rgba = Vec4::new(
							vec[0],
							vec[1],
							vec[2],
							if vec.len() >= 4 { vec[3] } else { 1.0 },
						);
					}
				}
			}
		}
		rgba
	}
}
