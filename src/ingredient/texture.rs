use std::{fs, rc::Rc};

use image::GenericImageView;
use russimp_ng::material::{self, DataContent};
use wgpu::{
	BindGroupEntry, BindGroupLayoutEntry, BufferUsages, Device, Extent3d, ImageCopyTexture,
	ImageDataLayout, Origin3d, Queue, Sampler, SamplerDescriptor, ShaderStages, Texture,
	TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
	TextureViewDescriptor,
};

use crate::{
	ingredient::{bindgroup::BindGroupIS, buffer::BufferIS},
	resources::texture_loader::ISTextureUnit,
};

pub struct TextureIS {}

pub struct DefaultTextures {
	pub white_tex: Texture,
	pub black_tex: Texture,
	pub normal_tex: Texture,
	pub sampler: Sampler,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniforms {
	pub diffuse_color: [f32; 4],
	pub specular_color: [f32; 4],
	pub ambient_color: [f32; 4],
	pub emissive_color: [f32; 4],
	pub params: [f32; 4],
}
impl TextureIS {
	pub fn load_pic(path: &str, device: &Device, queue: &Queue) -> BindGroupIS {
		let pic_byte = fs::read(path).unwrap();
		let pic_image = image::load_from_memory(&pic_byte).unwrap();

		let pic_rgb8 = pic_image.to_rgba8();
		let pic_dimention = pic_image.dimensions();

		Self::texdata2bindgroup(
			pic_dimention.0,
			pic_dimention.1,
			device,
			queue,
			&pic_rgb8,
			None,
		)
	}

	pub fn from_russimp_texture(
		russimp_tex: &material::Texture,
		device: &Device,
		queue: &Queue,
		texture_unit: &ISTextureUnit,
	) -> BindGroupIS {
		let (width, height, rgba_data) = Self::russimp_tex_to_rgba(russimp_tex);

		Self::texdata2bindgroup(width, height, device, queue, &rgba_data, Some(texture_unit))
	}

	fn texdata2bindgroup(
		width: u32,
		height: u32,
		device: &Device,
		queue: &Queue,
		data: &[u8],
		texture_unit: Option<&ISTextureUnit>,
	) -> BindGroupIS {
		let size = Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};
		let texture = device.create_texture(&TextureDescriptor {
			label: Some("russimp texture"),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format: TextureFormat::Rgba8UnormSrgb,
			usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
			view_formats: &[],
		});
		queue.write_texture(
			ImageCopyTexture {
				texture: &texture,
				mip_level: 0,
				origin: Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			data,
			ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * width),
				rows_per_image: Some(height),
			},
			size,
		);
		let view = texture.create_view(&TextureViewDescriptor::default());
		let sampler = device.create_sampler(&SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			address_mode_w: wgpu::AddressMode::Repeat,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		BindGroupIS::new(
			device,
			&[
				BindGroupLayoutEntry {
					binding: 0,
					visibility: ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 1,
					visibility: ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 2,
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
					resource: wgpu::BindingResource::TextureView(&view),
				},
				BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&sampler),
				},
				BindGroupEntry {
					binding: 2,
					resource: BufferIS::new(
						device,
						bytemuck::cast_slice(&texture_unit.unwrap().diffuse_color.to_array()),
						BufferUsages::UNIFORM,
					)
					.buffer
					.as_entire_binding(),
				},
			],
		)
	}
	pub fn create_material_bindgroup(
		device: &Device,
		queue: &Queue,
		texture_unit: &ISTextureUnit,
		defaults: &DefaultTextures,
	) -> BindGroupIS {
		let base_color_view = Self::upload_or_default(
			device,
			queue,
			&texture_unit.base_color,
			&defaults.white_tex,
			true,
		);
		let normal_view = Self::upload_or_default(
			device,
			queue,
			&texture_unit.normal,
			&defaults.normal_tex,
			false,
		);
		let roughness_view = Self::upload_or_default(
			device,
			queue,
			&texture_unit.roughness,
			&defaults.white_tex,
			false,
		);
		let metalness_view = Self::upload_or_default(
			device,
			queue,
			&texture_unit.metalness,
			&defaults.black_tex,
			false,
		);
		let ao_view = Self::upload_or_default(
			device,
			queue,
			&texture_unit.ambient_occlusion,
			&defaults.white_tex,
			false,
		);
		let emissive_view = Self::upload_or_default(
			device,
			queue,
			&texture_unit.emissive,
			&defaults.black_tex,
			true,
		);

		let material_uniforms = MaterialUniforms {
			diffuse_color: texture_unit.diffuse_color.to_array(),
			specular_color: texture_unit.specular_color.to_array(),
			ambient_color: texture_unit.ambient_color.to_array(),
			emissive_color: texture_unit.emissive_color.to_array(),
			params: [
				1.0,
				texture_unit.opacity_value,
				texture_unit.shininess,
				texture_unit.reflectivity,
			],
		};

		let uniform_buffer = BufferIS::new(
			device,
			bytemuck::cast_slice(&[material_uniforms]),
			BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		);

		BindGroupIS::new(
			device,
			&[
				BindGroupLayoutEntry {
					binding: 0,
					visibility: ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 1,
					visibility: ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 2,
					visibility: ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 3,
					visibility: ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 4,
					visibility: ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 5,
					visibility: ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 6,
					visibility: ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 7,
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
					resource: wgpu::BindingResource::TextureView(&base_color_view),
				},
				BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&defaults.sampler),
				},
				BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::TextureView(&normal_view),
				},
				BindGroupEntry {
					binding: 3,
					resource: wgpu::BindingResource::TextureView(&roughness_view),
				},
				BindGroupEntry {
					binding: 4,
					resource: wgpu::BindingResource::TextureView(&metalness_view),
				},
				BindGroupEntry {
					binding: 5,
					resource: wgpu::BindingResource::TextureView(&ao_view),
				},
				BindGroupEntry {
					binding: 6,
					resource: wgpu::BindingResource::TextureView(&emissive_view),
				},
				BindGroupEntry {
					binding: 7,
					resource: uniform_buffer.buffer.as_entire_binding(),
				},
			],
		)
	}

	fn russimp_tex_to_rgba(russimp_tex: &material::Texture) -> (u32, u32, Vec<u8>) {
		match &russimp_tex.data {
			DataContent::Texel(texels) => {
				let mut rgba = Vec::with_capacity(texels.len() * 4);
				for texel in texels {
					rgba.push(texel.r);
					rgba.push(texel.g);
					rgba.push(texel.b);
					rgba.push(texel.a);
				}
				(russimp_tex.width, russimp_tex.height, rgba)
			}
			DataContent::Bytes(bytes) => {
				let img = image::load_from_memory(bytes).unwrap();
				let rgba = img.to_rgba8();
				let (w, h) = img.dimensions();
				(w, h, rgba.into_raw())
			}
		}
	}
	fn upload_or_default(
		device: &Device,
		queue: &Queue,
		tex_opt: &Option<Rc<std::cell::RefCell<material::Texture>>>,
		default_tex: &Texture,
		is_srgb: bool,
	) -> TextureView {
		let format = if is_srgb {
			TextureFormat::Rgba8UnormSrgb
		} else {
			TextureFormat::Rgba8Unorm
		};
		match tex_opt {
			Some(tex_rc) => {
				let tex = tex_rc.borrow();
				let (w, h, data) = Self::russimp_tex_to_rgba(&tex);
				let gpu_tex = Self::upload_raw(device, queue, w, h, &data, format);
				gpu_tex.create_view(&TextureViewDescriptor::default())
			}
			None => default_tex.create_view(&TextureViewDescriptor::default()),
		}
	}

	fn upload_raw(
		device: &Device,
		queue: &Queue,
		width: u32,
		height: u32,
		data: &[u8],
		format: TextureFormat,
	) -> Texture {
		let size = Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};
		let texture = device.create_texture(&TextureDescriptor {
			label: Some("material texture"),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format,
			usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
			view_formats: &[],
		});
		queue.write_texture(
			ImageCopyTexture {
				texture: &texture,
				mip_level: 0,
				origin: Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			data,
			ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * width),
				rows_per_image: Some(height),
			},
			size,
		);
		texture
	}

	fn make_1x1_texture(
		device: &Device,
		queue: &Queue,
		rgba: &[u8; 4],
		format: TextureFormat,
	) -> Texture {
		let size = Extent3d {
			width: 1,
			height: 1,
			depth_or_array_layers: 1,
		};
		let texture = device.create_texture(&TextureDescriptor {
			label: Some("1x1 fallback texture"),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format,
			usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
			view_formats: &[],
		});
		queue.write_texture(
			ImageCopyTexture {
				texture: &texture,
				mip_level: 0,
				origin: Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			rgba,
			ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4),
				rows_per_image: Some(1),
			},
			size,
		);
		texture
	}

	pub fn create_default_textures(device: &Device, queue: &Queue) -> DefaultTextures {
		let white = Self::make_1x1_texture(
			device,
			queue,
			&[255, 255, 255, 255],
			TextureFormat::Rgba8UnormSrgb,
		);
		let black = Self::make_1x1_texture(
			device,
			queue,
			&[0, 0, 0, 255],
			TextureFormat::Rgba8UnormSrgb,
		);
		let normal = Self::make_1x1_texture(
			device,
			queue,
			&[128, 128, 255, 255],
			TextureFormat::Rgba8Unorm,
		);

		let sampler = device.create_sampler(&SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			address_mode_w: wgpu::AddressMode::Repeat,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		DefaultTextures {
			white_tex: white,
			black_tex: black,
			normal_tex: normal,
			sampler,
		}
	}

	pub fn default_white(device: &Device, queue: &Queue) -> (Texture, Sampler) {
		let size = Extent3d {
			width: 1,
			height: 1,
			depth_or_array_layers: 1,
		};
		let texture = device.create_texture(&TextureDescriptor {
			label: Some("default white texture"),
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format: TextureFormat::Rgba8UnormSrgb,
			usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
			view_formats: &[],
		});
		let white: [u8; 4] = [255, 255, 255, 255];
		queue.write_texture(
			ImageCopyTexture {
				texture: &texture,
				mip_level: 0,
				origin: Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			&white,
			ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4),
				rows_per_image: Some(1),
			},
			size,
		);
		let sampler = device.create_sampler(&SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			address_mode_w: wgpu::AddressMode::Repeat,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});
		(texture, sampler)
	}
}
