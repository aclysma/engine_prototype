use renderer::nodes::{
    RenderView, ViewSubmitNodes, FeatureSubmitNodes, FeatureCommandWriter, RenderFeatureIndex,
    FramePacket, RenderFeature, PrepareJob,
};
use crate::features::debug3d::{Debug3dRenderFeature, ExtractedDebugData, Debug3dDrawCall, Debug3dVertex, LineList2D};
use crate::phases::{OpaqueRenderPhase, PreUiRenderPhase};
use super::write::Debug3dCommandWriter;
use crate::render_contexts::{RenderJobWriteContext, RenderJobPrepareContext};
use renderer::vulkan::{VkBuffer, VkDeviceContext};
use ash::vk;
use renderer::assets::resources::{PipelineSwapchainInfo, DescriptorSetArc};
use minimum::resources::{DebugDraw3DDepthBehavior, LineList3D};
use renderer::assets::ResourceArc;
use renderer::vulkan::VkBufferRaw;

pub struct Debug3dPrepareJobImpl {
    device_context: VkDeviceContext,
    pipeline_info_3d: PipelineSwapchainInfo,
    pipeline_info_3d_no_depth: PipelineSwapchainInfo,
    pipeline_info_2d: PipelineSwapchainInfo,
    dyn_resource_allocator: renderer::assets::DynResourceAllocatorSet,
    descriptor_set_per_view_3d: Vec<DescriptorSetArc>,
    descriptor_set_2d: DescriptorSetArc,
    extracted_debug_data: ExtractedDebugData,
}

impl Debug3dPrepareJobImpl {
    pub(super) fn new(
        device_context: VkDeviceContext,
        pipeline_info_3d: PipelineSwapchainInfo,
        pipeline_info_3d_no_depth: PipelineSwapchainInfo,
        pipeline_info_2d: PipelineSwapchainInfo,
        dyn_resource_allocator: renderer::assets::DynResourceAllocatorSet,
        descriptor_set_per_view_3d: Vec<DescriptorSetArc>,
        descriptor_set_2d: DescriptorSetArc,
        extracted_debug_data: ExtractedDebugData,
    ) -> Self {
        Debug3dPrepareJobImpl {
            device_context,
            pipeline_info_3d,
            pipeline_info_3d_no_depth,
            pipeline_info_2d,
            dyn_resource_allocator,
            descriptor_set_per_view_3d,
            descriptor_set_2d,
            extracted_debug_data,
        }
    }
}

impl PrepareJob<RenderJobPrepareContext, RenderJobWriteContext> for Debug3dPrepareJobImpl {
    fn prepare(
        self: Box<Self>,
        _prepare_context: &RenderJobPrepareContext,
        _frame_packet: &FramePacket,
        views: &[&RenderView],
    ) -> (
        Box<dyn FeatureCommandWriter<RenderJobWriteContext>>,
        FeatureSubmitNodes,
    ) {
        //
        // Gather the raw draw data
        //
        let line_lists_3d = &self.extracted_debug_data.line_lists_3d;
        let mut draw_calls_3d = Vec::with_capacity(line_lists_3d.len());
        let mut draw_calls_3d_no_depth = Vec::with_capacity(line_lists_3d.len());

        let mut vertex_list_3d: Vec<Debug3dVertex> = vec![];
        let mut vertex_list_3d_no_depth: Vec<Debug3dVertex> = vec![];
        for line_list in line_lists_3d {
            match line_list.depth_behavior {
                DebugDraw3DDepthBehavior::Normal => Debug3dPrepareJobImpl::add_line_list_3d(&mut vertex_list_3d, &mut draw_calls_3d, line_list),
                DebugDraw3DDepthBehavior::NoDepthTest => Debug3dPrepareJobImpl::add_line_list_3d(&mut vertex_list_3d_no_depth, &mut draw_calls_3d_no_depth, line_list)
            }
        }

        let vertex_buffer_3d = self.create_vertex_buffer(&mut draw_calls_3d, vertex_list_3d);
        let vertex_buffer_3d_no_depth = self.create_vertex_buffer(&mut draw_calls_3d_no_depth, vertex_list_3d_no_depth);

        let line_lists_2d = &self.extracted_debug_data.line_lists_2d;
        let mut draw_calls_2d = Vec::with_capacity(line_lists_3d.len());
        let mut vertex_list_2d: Vec<Debug3dVertex> = vec![];

        for line_list in line_lists_2d {
            Debug3dPrepareJobImpl::add_line_list_2d(&mut vertex_list_2d, &mut draw_calls_2d, line_list);
        }

        let vertex_buffer_2d = self.create_vertex_buffer(&mut draw_calls_2d, vertex_list_2d);

        //
        // Submit a single node for each view
        // TODO: Submit separate nodes for transparency
        //
        let mut submit_nodes = FeatureSubmitNodes::default();
        for view in views {
            let mut view_submit_nodes =
                ViewSubmitNodes::new(self.feature_index(), view.render_phase_mask());
            view_submit_nodes.add_submit_node::<OpaqueRenderPhase>(0, 0, 0.0);
            view_submit_nodes.add_submit_node::<PreUiRenderPhase>(1, 0, 0.0);
            view_submit_nodes.add_submit_node::<PreUiRenderPhase>(2, 0, 0.0);
            submit_nodes.add_submit_nodes_for_view(view, view_submit_nodes);
        }

        let writer = Box::new(Debug3dCommandWriter {
            draw_calls_3d,
            vertex_buffer_3d,
            draw_calls_3d_no_depth,
            vertex_buffer_3d_no_depth,
            draw_calls_2d,
            vertex_buffer_2d,
            pipeline_info_3d: self.pipeline_info_3d,
            pipeline_info_3d_no_depth: self.pipeline_info_3d_no_depth,
            pipeline_info_2d: self.pipeline_info_2d,
            descriptor_set_per_view_3d: self.descriptor_set_per_view_3d,
            descriptor_set_2d: self.descriptor_set_2d,
        });

        (writer, submit_nodes)
    }

    fn feature_debug_name(&self) -> &'static str {
        Debug3dRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        Debug3dRenderFeature::feature_index()
    }
}


impl Debug3dPrepareJobImpl {
    fn add_line_list_3d(
        vertex_list: &mut Vec<Debug3dVertex>,
        draw_calls: &mut Vec<Debug3dDrawCall>,
        line_list: &LineList3D,
    ) {
        let vertex_buffer_first_element = vertex_list.len() as u32;

        for vertex_pos in &line_list.points {
            vertex_list.push(Debug3dVertex {
                pos: (*vertex_pos).into(),
                color: line_list.color.into(),
            });
        }

        let draw_call = Debug3dDrawCall {
            first_element: vertex_buffer_first_element,
            count: line_list.points.len() as u32,
        };

        draw_calls.push(draw_call);
    }

    fn add_line_list_2d(
        vertex_list: &mut Vec<Debug3dVertex>,
        draw_calls: &mut Vec<Debug3dDrawCall>,
        line_list: &LineList2D,
    ) {
        let vertex_buffer_first_element = vertex_list.len() as u32;

        for vertex_pos in &line_list.points {
            vertex_list.push(Debug3dVertex {
                pos: (*vertex_pos).extend(0.0).into(),
                color: line_list.color.into(),
            });
        }

        let draw_call = Debug3dDrawCall {
            first_element: vertex_buffer_first_element,
            count: line_list.points.len() as u32,
        };

        draw_calls.push(draw_call);
    }

    fn create_vertex_buffer(
        &self,
        draw_calls: &mut Vec<Debug3dDrawCall>,
        vertex_list: Vec<Debug3dVertex>
    ) -> Option<ResourceArc<VkBufferRaw>> {
        // We would probably want to support multiple buffers at some point
        if !draw_calls.is_empty() {
            let vertex_buffer_size =
                vertex_list.len() as u64 * std::mem::size_of::<Debug3dVertex>() as u64;
            let mut vertex_buffer = VkBuffer::new(
                &self.device_context,
                vk_mem::MemoryUsage::CpuToGpu,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                vertex_buffer_size,
            ).unwrap();

            vertex_buffer
                .write_to_host_visible_buffer(vertex_list.as_slice())
                .unwrap();

            Some(self.dyn_resource_allocator.insert_buffer(vertex_buffer))
        } else {
            None
        }
    }
}
