use crate::render_contexts::{RenderJobExtractContext, RenderJobPrepareContext, RenderJobWriteContext};
use atelier_assets::loader::handle::Handle;
use crate::features::imgui::extract::ImGuiExtractJobImpl;
use renderer::vulkan::VkDeviceContext;
use renderer::assets::DescriptorSetAllocatorRef;
use renderer::assets::PipelineSwapchainInfo;
use renderer::nodes::ExtractJob;
use renderer::nodes::RenderFeature;
use renderer::nodes::RenderFeatureIndex;
use std::convert::TryInto;
use minimum::game::imgui::ImguiDrawData;
use ash::vk::Extent2D;
use renderer::assets::{ImageViewResource, ResourceArc};
use renderer::assets::MaterialAsset;

mod extract;
mod prepare;
mod write;

pub fn create_imgui_extract_job(
    device_context: VkDeviceContext,
    descriptor_set_allocator: DescriptorSetAllocatorRef,
    pipeline_info: PipelineSwapchainInfo,
    extents: Extent2D,
    imgui_material: &Handle<MaterialAsset>,
    font_atlas: ResourceArc<ImageViewResource>,
) -> Box<dyn ExtractJob<RenderJobExtractContext, RenderJobPrepareContext, RenderJobWriteContext>> {
    Box::new(ImGuiExtractJobImpl::new(
        device_context,
        descriptor_set_allocator,
        pipeline_info,
        extents,
        imgui_material,
        font_atlas,
    ))
}

/// Per-pass "global" data
#[derive(Clone, Debug, Copy)]
struct ImGuiUniformBufferObject {
    // View and projection matrices
    view_proj: [[f32; 4]; 4],
}

/// Vertex format for vertices sent to the GPU
#[derive(Clone, Debug, Copy)]
#[repr(C)]
pub struct ImGuiVertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

renderer::declare_render_feature!(ImGuiRenderFeature, IMGUI_RENDER_FEATURE);

pub(self) struct ExtractedImGuiData {
    imgui_draw_data: Option<ImguiDrawData>,
}

#[derive(Debug)]
struct ImGuiDrawCall {
    first_element: u32,
    count: u32,
}
