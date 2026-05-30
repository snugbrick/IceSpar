use wgpu::{
	BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
	Device,
};

pub struct BindGroupIS {
	pub bindgrouplayout: BindGroupLayout,
	pub bindgroup: BindGroup,
}
impl BindGroupIS {
	pub fn new(
		device: &Device,
		bind_group_layout_entry: &[BindGroupLayoutEntry],
		bind_group_entry: &[BindGroupEntry],
	) -> Self {
		let bindgroup_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: Some("bind group"),
			entries: bind_group_layout_entry,
		});
		let bindgrup = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("bind group"),
			layout: &bindgroup_layout,
			entries: bind_group_entry,
		});
		Self {
			bindgrouplayout: bindgroup_layout,
			bindgroup: bindgrup,
		}
	}
}
