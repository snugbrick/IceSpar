use std::{cell::RefCell, rc::Rc};

use glam::Vec4;
use russimp_ng::{
	material::{Material, MaterialProperty, PropertyTypeInfo, Texture, TextureType},
	mesh::Mesh,
};

pub struct ISTextureUnit {
	pub material_index: usize,
	pub mesh_index: usize,
	pub base_color: Option<Rc<RefCell<Texture>>>,
	pub normal: Option<Rc<RefCell<Texture>>>,
	pub roughness: Option<Rc<RefCell<Texture>>>,
	pub metalness: Option<Rc<RefCell<Texture>>>,
	pub ambient_occlusion: Option<Rc<RefCell<Texture>>>,
	pub emissive: Option<Rc<RefCell<Texture>>>,
	pub specular: Option<Rc<RefCell<Texture>>>,
	pub ambient: Option<Rc<RefCell<Texture>>>,
	pub height: Option<Rc<RefCell<Texture>>>,
	pub shininess_tex: Option<Rc<RefCell<Texture>>>,
	pub opacity: Option<Rc<RefCell<Texture>>>,
	pub lightmap: Option<Rc<RefCell<Texture>>>,
	pub reflection: Option<Rc<RefCell<Texture>>>,
	pub displacement: Option<Rc<RefCell<Texture>>>,
	pub sheen: Option<Rc<RefCell<Texture>>>,
	pub clearcoat: Option<Rc<RefCell<Texture>>>,
	pub transmission: Option<Rc<RefCell<Texture>>>,
	pub diffuse_color: Vec4,
	pub specular_color: Vec4,
	pub ambient_color: Vec4,
	pub emissive_color: Vec4,
	pub shininess: f32,
	pub opacity_value: f32,
	pub reflectivity: f32,
}

impl ISTextureUnit {
	pub fn from_mesh(meshes: &[Mesh], mesh_index: usize, materials: &[Material]) -> Self {
		let material_index = meshes[mesh_index].material_index as usize;
		let material = &materials[material_index];
		let textures = &material.textures;
		let properties = &material.properties;

		Self {
			material_index,
			mesh_index,
			base_color: textures
				.get(&TextureType::BaseColor)
				.or_else(|| textures.get(&TextureType::Diffuse))
				.cloned(),
			normal: textures
				.get(&TextureType::NormalCamera)
				.or_else(|| textures.get(&TextureType::Normals))
				.cloned(),
			roughness: textures.get(&TextureType::Roughness).cloned(),
			metalness: textures.get(&TextureType::Metalness).cloned(),
			ambient_occlusion: textures.get(&TextureType::AmbientOcclusion).cloned(),
			emissive: textures
				.get(&TextureType::EmissionColor)
				.or_else(|| textures.get(&TextureType::Emissive))
				.cloned(),
			specular: textures.get(&TextureType::Specular).cloned(),
			ambient: textures.get(&TextureType::Ambient).cloned(),
			height: textures.get(&TextureType::Height).cloned(),
			shininess_tex: textures.get(&TextureType::Shininess).cloned(),
			opacity: textures.get(&TextureType::Opacity).cloned(),
			lightmap: textures.get(&TextureType::LightMap).cloned(),
			reflection: textures.get(&TextureType::Reflection).cloned(),
			displacement: textures.get(&TextureType::Displacement).cloned(),
			sheen: textures.get(&TextureType::Sheen).cloned(),
			clearcoat: textures.get(&TextureType::ClearCoat).cloned(),
			transmission: textures.get(&TextureType::Transmission).cloned(),
			diffuse_color: Self::get_color_property(properties, "$clr.diffuse"),
			specular_color: Self::get_color_property(properties, "$clr.specular"),
			ambient_color: Self::get_color_property(properties, "$clr.ambient"),
			emissive_color: Self::get_color_property(properties, "$clr.emissive"),
			shininess: Self::get_float_property(properties, "$mat.shininess", 1.0),
			opacity_value: Self::get_float_property(properties, "$mat.opacity", 1.0),
			reflectivity: Self::get_float_property(properties, "$mat.reflectivity", 0.0),
		}
	}

	fn get_color_property(properties: &[MaterialProperty], key: &str) -> Vec4 {
		for prop in properties {
			if prop.key == key
				&& let PropertyTypeInfo::FloatArray(vec) = &prop.data
				&& vec.len() >= 3
			{
				return Vec4::new(
					vec[0],
					vec[1],
					vec[2],
					if vec.len() >= 4 { vec[3] } else { 1.0 },
				);
			}
		}
		match key {
			"$clr.diffuse" => Vec4::new(1.0, 1.0, 1.0, 1.0),
			"$clr.specular" => Vec4::new(1.0, 1.0, 1.0, 1.0),
			"$clr.ambient" => Vec4::new(0.0, 0.0, 0.0, 1.0),
			"$clr.emissive" => Vec4::new(0.0, 0.0, 0.0, 1.0),
			_ => Vec4::ONE,
		}
	}

	fn get_float_property(properties: &[MaterialProperty], key: &str, default: f32) -> f32 {
		for prop in properties {
			if prop.key == key
				&& let PropertyTypeInfo::FloatArray(vec) = &prop.data
			{
				return vec[0];
			}
		}
		default
	}
}
