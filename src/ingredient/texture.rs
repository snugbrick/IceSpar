use std::fs;

use image::GenericImageView;
use russimp_ng::material::{DataContent, Texture as RussimpTexture};
use wgpu::{
	BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, Device, Extent3d,
	ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Sampler, SamplerDescriptor, ShaderStages,
	TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
	TextureViewDescriptor,
};

use crate::ingredient::bindgroup::BindGroupIS;

pub struct TextureIS {
	pub texture_layout: BindGroupLayout,
	pub texture: BindGroup,
}

pub struct TextureViewSampler {
	pub view: TextureView,
	pub sampler: Sampler,
}

impl TextureIS {
	pub fn load_pic(path: &str, device: &Device, queue: &Queue) -> Self {
		let pic_byte = fs::read(path).unwrap();
		let pic_image = image::load_from_memory(&pic_byte).unwrap();

		let pic_rgb8 = pic_image.to_rgba8();
		let pic_dimention = pic_image.dimensions();

		let pic_size = Extent3d {
			width: pic_dimention.0,
			height: pic_dimention.1,
			depth_or_array_layers: 1,
		};
		let texture_4queue = device.create_texture(&TextureDescriptor {
			label: Some("a texture"),
			size: pic_size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format: TextureFormat::Etc2Rgba8UnormSrgb,
			usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
			view_formats: &[],
		});
		queue.write_texture(
			ImageCopyTexture {
				texture: &texture_4queue,
				mip_level: 0,
				origin: Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			&pic_rgb8,
			ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * pic_dimention.0),
				rows_per_image: Some(pic_dimention.1),
			},
			pic_size,
		);
		let image_view = texture_4queue.create_view(&TextureViewDescriptor::default());
		let image_sampler = device.create_sampler(&SamplerDescriptor {
			label: Some("image sampler"),
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});
		let texture_bindgroup = BindGroupIS::new(
			&device,
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
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
			&[
				BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&image_view),
				},
				BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&image_sampler),
				},
			],
		);

		Self {
			texture_layout: texture_bindgroup.bindgrouplayout,
			texture: texture_bindgroup.bindgroup,
		}
	}

	pub fn from_russimp_texture(
		russimp_tex: &RussimpTexture,
		device: &Device,
		queue: &Queue,
	) -> TextureViewSampler {
		let (width, height, rgba_data) = match &russimp_tex.data {
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
		};

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
			&rgba_data,
			ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * width),
				rows_per_image: Some(height),
			},
			size,
		);
		let view = texture.create_view(&TextureViewDescriptor::default());
		let sampler = Self::make_sampler(device);
		TextureViewSampler { view, sampler }
	}
	pub fn default_white(device: &Device, queue: &Queue) -> (wgpu::Texture, Sampler) {
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
		let sampler = Self::make_sampler(device);
		(texture, sampler)
	}

	fn make_sampler(device: &Device) -> Sampler {
		device.create_sampler(&SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			address_mode_w: wgpu::AddressMode::Repeat,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		})
	}
}
