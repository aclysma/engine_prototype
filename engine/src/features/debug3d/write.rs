use crate::features::debug3d::{Debug3dRenderFeature, Debug3dDrawCall};
use renderer::nodes::{
    RenderFeatureIndex, RenderPhaseIndex, RenderFeature, SubmitNodeId, FeatureCommandWriter, RenderView,
};
use crate::render_contexts::RenderJobWriteContext;
use renderer::vulkan::VkBufferRaw;
use renderer::assets::resources::{ResourceArc, PipelineSwapchainInfo, DescriptorSetArc};
use ash::vk;
use ash::version::DeviceV1_0;

pub struct Debug3dCommandWriter {
    pub(super) vertex_buffer_3d: Option<ResourceArc<VkBufferRaw>>,
    pub(super) draw_calls_3d: Vec<Debug3dDrawCall>,
    pub(super) vertex_buffer_3d_no_depth: Option<ResourceArc<VkBufferRaw>>,
    pub(super) draw_calls_3d_no_depth: Vec<Debug3dDrawCall>,
    pub(super) vertex_buffer_2d: Option<ResourceArc<VkBufferRaw>>,
    pub(super) draw_calls_2d: Vec<Debug3dDrawCall>,
    pub(super) pipeline_info_3d: PipelineSwapchainInfo,
    pub(super) pipeline_info_3d_no_depth: PipelineSwapchainInfo,
    pub(super) pipeline_info_2d: PipelineSwapchainInfo,
    pub(super) descriptor_set_per_view_3d: Vec<DescriptorSetArc>,
    pub(super) descriptor_set_2d: DescriptorSetArc,
}

impl FeatureCommandWriter<RenderJobWriteContext> for Debug3dCommandWriter {
    fn apply_setup(
        &self,
        _write_context: &mut RenderJobWriteContext,
        _view: &RenderView,
        _render_phase_index: RenderPhaseIndex,
    ) {
        // Nothing here, render_element gets called once per phase so there is no advantage to
        // doing setup here
    }

    fn render_element(
        &self,
        write_context: &mut RenderJobWriteContext,
        view: &RenderView,
        _render_phase_index: RenderPhaseIndex,
        index: SubmitNodeId,
    ) {
        let logical_device = write_context.device_context.device();
        let command_buffer = write_context.command_buffer;

        // The prepare phase emits a single node which will draw everything. In the future it might
        // emit a node per draw call that uses transparency
        if index == 0 {
            if let Some(vertex_buffer_3d) = self.vertex_buffer_3d.as_ref() {
                unsafe {
                    logical_device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_info_3d.pipeline.get_raw().pipelines[0],
                    );

                    // Bind per-pass data (UBO with view/proj matrix, sampler)
                    logical_device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_info_3d.pipeline_layout.get_raw().pipeline_layout,
                        0,
                        &[self.descriptor_set_per_view_3d[view.view_index() as usize].get()],
                        &[],
                    );

                    logical_device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0, // first binding
                        &[vertex_buffer_3d.get_raw().buffer],
                        &[0], // offsets
                    );

                    for draw_call in &self.draw_calls_3d {
                        logical_device.cmd_draw(
                            command_buffer,
                            draw_call.count as u32,
                            1,
                            draw_call.first_element as u32,
                            0,
                        );
                    }
                }
            }
        } else if index == 1 {
            if let Some(vertex_buffer_3d_no_depth) = self.vertex_buffer_3d_no_depth.as_ref() {
                unsafe {
                    logical_device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_info_3d_no_depth.pipeline.get_raw().pipelines[0],
                    );

                    // Bind per-pass data (UBO with view/proj matrix, sampler)
                    logical_device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_info_3d_no_depth.pipeline_layout.get_raw().pipeline_layout,
                        0,
                        &[self.descriptor_set_per_view_3d[view.view_index() as usize].get()],
                        &[],
                    );

                    logical_device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0, // first binding
                        &[vertex_buffer_3d_no_depth.get_raw().buffer],
                        &[0], // offsets
                    );

                    for draw_call in &self.draw_calls_3d_no_depth {
                        logical_device.cmd_draw(
                            command_buffer,
                            draw_call.count as u32,
                            1,
                            draw_call.first_element as u32,
                            0,
                        );
                    }
                }
            }
        } else if index == 2 {
            if let Some(vertex_buffer_2d) = self.vertex_buffer_2d.as_ref() {
                unsafe {
                    logical_device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_info_2d.pipeline.get_raw().pipelines[0],
                    );

                    // Bind per-pass data (UBO with view/proj matrix, sampler)
                    logical_device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_info_2d.pipeline_layout.get_raw().pipeline_layout,
                        0,
                        &[self.descriptor_set_2d.get()],
                        &[],
                    );

                    logical_device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0, // first binding
                        &[vertex_buffer_2d.get_raw().buffer],
                        &[0], // offsets
                    );

                    for draw_call in &self.draw_calls_2d {
                        logical_device.cmd_draw(
                            command_buffer,
                            draw_call.count as u32,
                            1,
                            draw_call.first_element as u32,
                            0,
                        );
                    }
                }
            }
        }
    }

    fn revert_setup(
        &self,
        _write_context: &mut RenderJobWriteContext,
        _view: &RenderView,
        _render_phase_index: RenderPhaseIndex,
    ) {
    }

    fn feature_debug_name(&self) -> &'static str {
        Debug3dRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        Debug3dRenderFeature::feature_index()
    }
}
