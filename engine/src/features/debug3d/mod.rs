use crate::render_contexts::{RenderJobExtractContext, RenderJobPrepareContext, RenderJobWriteContext};
use atelier_assets::loader::handle::Handle;
use crate::features::debug3d::extract::Debug3dExtractJobImpl;
use renderer::vulkan::VkDeviceContext;
use renderer::assets::DescriptorSetAllocatorRef;
use renderer::assets::PipelineSwapchainInfo;
use renderer::nodes::ExtractJob;
use renderer::nodes::RenderFeature;
use renderer::nodes::RenderFeatureIndex;
use std::convert::TryInto;
use renderer::assets::MaterialAsset;

mod extract;
mod prepare;
mod write;

pub use minimum::game::resources::LineList3D;
pub use minimum::game::resources::DebugDraw3DResource;
pub use minimum::game::resources::LineList2D;
pub use minimum::game::resources::DebugDraw2DResource;
use ash::vk::Extent2D;

pub fn create_debug3d_extract_job(
    device_context: VkDeviceContext,
    descriptor_set_allocator: DescriptorSetAllocatorRef,
    extents: Extent2D,
    pipeline_info_3d: PipelineSwapchainInfo,
    pipeline_info_3d_no_depth: PipelineSwapchainInfo,
    pipeline_info_2d: PipelineSwapchainInfo,
    debug_material_3d: &Handle<MaterialAsset>,
    debug_material_3d_no_depth: &Handle<MaterialAsset>,
    debug_material_2d: &Handle<MaterialAsset>,
) -> Box<dyn ExtractJob<RenderJobExtractContext, RenderJobPrepareContext, RenderJobWriteContext>> {
    Box::new(Debug3dExtractJobImpl::new(
        device_context,
        descriptor_set_allocator,
        extents,
        pipeline_info_3d,
        pipeline_info_3d_no_depth,
        pipeline_info_2d,
        debug_material_3d.clone(),
        debug_material_3d_no_depth.clone(),
        debug_material_2d.clone(),
    ))
}

/// Per-pass "global" data
#[derive(Clone, Debug, Copy)]
struct Debug3dUniformBufferObject {
    // View and projection matrices
    view_proj: [[f32; 4]; 4],
}

/// Vertex format for vertices sent to the GPU
#[derive(Clone, Debug, Copy)]
#[repr(C)]
pub struct Debug3dVertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

renderer::declare_render_feature!(Debug3dRenderFeature, DEBUG_3D_RENDER_FEATURE);

pub(self) struct ExtractedDebugData {
    line_lists_3d: Vec<LineList3D>,
    line_lists_2d: Vec<LineList2D>,
}

#[derive(Debug)]
struct Debug3dDrawCall {
    first_element: u32,
    count: u32,
}
