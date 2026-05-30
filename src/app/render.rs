use wgpu::{
	BindGroup, CommandEncoder, CommandEncoderDescriptor, Queue, RenderPassDescriptor, SurfaceTexture,
	TextureView, TextureViewDescriptor,
};

use crate::{
	app::app::App,
	ingredient::{buffer::VIBufferFromMesh, pipeline::PipelineIS},
};

pub struct RenderPreparation {
	pub frame: SurfaceTexture,
	pub view: TextureView,
	pub encoder: CommandEncoder,

	pub depth_view: TextureView,
}
impl RenderPreparation {
	pub fn prepare_render(app: &mut App) -> Self {
		let frame = app.surface.get_current_texture().unwrap();
		let view = frame.texture.create_view(&Default::default());
		let encoder = app
			.device
			.create_command_encoder(&CommandEncoderDescriptor {
				label: Some("just a encoder"),
			});
		Self {
			frame: frame,
			view: view,
			encoder: encoder,

			depth_view: app
				.depth_texture
				.as_ref()
				.expect("enable_depth_test not called")
				.create_view(&TextureViewDescriptor::default()),
		}
	}
	pub fn record_renderpass<'a>(
		&mut self,
		pipeline: &PipelineIS,
		mesh_buffers: &[VIBufferFromMesh<'a>],
		bindgroup: &[&BindGroup],
	) {
		let render_des = RenderPassDescriptor {
			label: Some("Render Pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &self.view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color {
						r: 0.1,
						g: 0.2,
						b: 0.3,
						a: 1.0,
					}),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
				view: &self.depth_view,
				depth_ops: Some(wgpu::Operations {
					load: wgpu::LoadOp::Clear(1.0),
					store: wgpu::StoreOp::Store,
				}),
				stencil_ops: None,
			}),
			occlusion_query_set: None,
			timestamp_writes: None,
		};
		let mut renderpass = self.encoder.begin_render_pass(&render_des);
		renderpass.set_pipeline(&pipeline.pipeline);
		for bind_group_index in 0..bindgroup.len() {
			renderpass.set_bind_group(bind_group_index as u32, &bindgroup[bind_group_index], &[]);
		}
		for vi_buffer in mesh_buffers.iter() {
			renderpass.set_vertex_buffer(0, vi_buffer.vertex_buffer.buffer.slice(..));
			renderpass.set_index_buffer(vi_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
			renderpass.draw_indexed(0..vi_buffer.index_count, 0, 0..1);
		}
	}
	pub fn present(self, queue: &Queue) {
		let fir_graph = self.encoder.finish();
		queue.submit(Some(fir_graph));
		self.frame.present();
	}
}
