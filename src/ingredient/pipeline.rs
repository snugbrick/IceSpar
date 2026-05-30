use std::num::NonZero;

use wgpu::{
	BindGroupLayout, DepthStencilState, Device, FragmentState, MultisampleState, PipelineLayout,
	PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, VertexState,
};

pub struct PipelineIS {
	pub pipeline: RenderPipeline,
}

pub struct PipelineBuilderIS {
	pub pipeline_layout: PipelineLayout,

	pub primitive_state: PrimitiveState,
	pub depth_stencil_state: Option<DepthStencilState>,
	pub multisample_state: MultisampleState,
	pub multiview_state: Option<NonZero<u32>>,
}
impl PipelineIS {
	pub fn begin_layout(
		device: &Device,
		bind_group_layout: &[&BindGroupLayout],
	) -> PipelineBuilderIS {
		PipelineBuilderIS {
			pipeline_layout: device.create_pipeline_layout(&PipelineLayoutDescriptor {
				label: Some("pipeline layout"),
				bind_group_layouts: bind_group_layout,
				push_constant_ranges: &[],
			}),
			primitive_state: PrimitiveState::default(),
			depth_stencil_state: None,
			multisample_state: MultisampleState::default(),
			multiview_state: None,
		}
	}
}
impl PipelineBuilderIS {
	pub fn get_pipeline(
		self,
		device: &Device,
		vertex_state: VertexState,
		fragement_state: FragmentState,
	) -> PipelineIS {
		PipelineIS {
			pipeline: device.create_render_pipeline(&RenderPipelineDescriptor {
				label: Some("pipeline"),
				layout: Some(&self.pipeline_layout),
				vertex: vertex_state,
				fragment: Some(fragement_state),

				primitive: self.primitive_state,
				depth_stencil: self.depth_stencil_state,
				multisample: self.multisample_state,
				multiview: self.multiview_state,
			}),
		}
	}
	pub fn set_primitive_state(mut self, primitive_state: PrimitiveState) -> Self {
		self.primitive_state = primitive_state;
		self
	}
	pub fn set_depthstencil_state(mut self, depth_stencil_state: DepthStencilState) -> Self {
		self.depth_stencil_state = Some(depth_stencil_state);
		self
	}
	pub fn set_multisample_state(mut self, multisample_state: MultisampleState) -> Self {
		self.multisample_state = multisample_state;
		self
	}
	pub fn set_multiview_state(mut self, multiview_state: NonZero<u32>) -> Self {
		self.multiview_state = Some(multiview_state);
		self
	}
}
