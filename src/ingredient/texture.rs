use std::fs;

use image::GenericImageView;
use wgpu::{
	BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, Device, Extent3d,
	ImageCopyTexture, ImageDataLayout, Origin3d, Queue, SamplerDescriptor, ShaderStages,
	TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};

use crate::ingredient::bindgroup::BindGroupIS;

pub struct TextureIS {
	pub texture_layout: BindGroupLayout,
	pub texture: BindGroup,
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
}
