use crate::features::debug3d::{
    ExtractedDebugData, Debug3dRenderFeature, DebugDraw2DResource, DebugDraw3DResource,
    Debug3dUniformBufferObject,
};
use crate::render_contexts::{RenderJobExtractContext, RenderJobWriteContext, RenderJobPrepareContext};
use renderer::nodes::{
    FramePacket, RenderView, PrepareJob, RenderFeatureIndex, RenderFeature, ExtractJob,
};
use crate::features::debug3d::prepare::Debug3dPrepareJobImpl;
use renderer::vulkan::VkDeviceContext;
use renderer::assets::resources::{PipelineSwapchainInfo, DescriptorSetAllocatorRef};
use atelier_assets::loader::handle::Handle;
use renderer::assets::MaterialAsset;
use ash::vk::Extent2D;

// This is almost copy-pasted from glam. I wanted to avoid pulling in the entire library for a
// single function
pub fn orthographic_rh_gl(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> [[f32; 4]; 4] {
    let a = 2.0 / (right - left);
    let b = 2.0 / (top - bottom);
    let c = -2.0 / (far - near);
    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);
    let tz = -(far + near) / (far - near);

    [
        [a, 0.0, 0.0, 0.0],
        [0.0, b, 0.0, 0.0],
        [0.0, 0.0, c, 0.0],
        [tx, ty, tz, 1.0],
    ]
}

pub struct Debug3dExtractJobImpl {
    device_context: VkDeviceContext,
    descriptor_set_allocator: DescriptorSetAllocatorRef,
    extents: Extent2D,
    pipeline_info_3d: PipelineSwapchainInfo,
    pipeline_info_3d_no_depth: PipelineSwapchainInfo,
    pipeline_info_2d: PipelineSwapchainInfo,
    debug_material_3d: Handle<MaterialAsset>,
    debug_material_3d_no_depth: Handle<MaterialAsset>,
    debug_material_2d: Handle<MaterialAsset>,
}

impl Debug3dExtractJobImpl {
    pub fn new(
        device_context: VkDeviceContext,
        descriptor_set_allocator: DescriptorSetAllocatorRef,
        extents: Extent2D,
        pipeline_info_3d: PipelineSwapchainInfo,
        pipeline_info_3d_no_depth: PipelineSwapchainInfo,
        pipeline_info_2d: PipelineSwapchainInfo,
        debug_material_3d: Handle<MaterialAsset>,
        debug_material_3d_no_depth: Handle<MaterialAsset>,
        debug_material_2d: Handle<MaterialAsset>,
    ) -> Self {
        Debug3dExtractJobImpl {
            device_context,
            descriptor_set_allocator,
            extents,
            pipeline_info_3d,
            pipeline_info_3d_no_depth,
            pipeline_info_2d,
            debug_material_3d,
            debug_material_3d_no_depth,
            debug_material_2d,
        }
    }
}

impl ExtractJob<RenderJobExtractContext, RenderJobPrepareContext, RenderJobWriteContext>
    for Debug3dExtractJobImpl
{
    fn extract(
        mut self: Box<Self>,
        extract_context: &RenderJobExtractContext,
        _frame_packet: &FramePacket,
        views: &[&RenderView],
    ) -> Box<dyn PrepareJob<RenderJobPrepareContext, RenderJobWriteContext>> {
        let dyn_resource_allocator = extract_context
            .resource_manager
            .create_dyn_resource_allocator_set();
        let layout =
            extract_context
                .resource_manager
                .get_descriptor_set_info(&self.debug_material_3d, 0, 0);

        let per_view_descriptor_sets_3d: Vec<_> = views
            .iter()
            .map(|view| {
                let debug3d_view = Debug3dUniformBufferObject {
                    view_proj: (view.projection_matrix() * view.view_matrix()).to_cols_array_2d(),
                };

                let mut descriptor_set = self
                    .descriptor_set_allocator
                    .create_dyn_descriptor_set_uninitialized(&layout.descriptor_set_layout)
                    .unwrap();
                descriptor_set.set_buffer_data(0, &debug3d_view);
                descriptor_set
                    .flush(&mut self.descriptor_set_allocator)
                    .unwrap();
                descriptor_set.descriptor_set().clone()
            })
            .collect();

        let descriptor_set_2d = {
            // https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
            let vulkan_projection_correction =
                glam::Mat4::from_scale(glam::Vec3::new(1.0, -1.0, 0.5))
                    * glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 1.0));
            let view_proj = vulkan_projection_correction
                * glam::Mat4::orthographic_rh_gl(
                    0.0,
                    self.extents.width as f32,
                    self.extents.height as f32,
                    0.0,
                    -100.0,
                    100.0,
                );
            let debug3d_view = Debug3dUniformBufferObject {
                view_proj: view_proj.to_cols_array_2d(),
            };

            let mut descriptor_set = self
                .descriptor_set_allocator
                .create_dyn_descriptor_set_uninitialized(&layout.descriptor_set_layout)
                .unwrap();
            descriptor_set.set_buffer_data(0, &debug3d_view);
            descriptor_set
                .flush(&mut self.descriptor_set_allocator)
                .unwrap();
            descriptor_set.descriptor_set().clone()
        };

        let line_lists_2d = extract_context
            .resources
            .get_mut::<DebugDraw2DResource>()
            .unwrap()
            .take_line_lists();

        let line_lists_3d = extract_context
            .resources
            .get_mut::<DebugDraw3DResource>()
            .unwrap()
            .take_line_lists();

        Box::new(Debug3dPrepareJobImpl::new(
            self.device_context,
            self.pipeline_info_3d,
            self.pipeline_info_3d_no_depth,
            self.pipeline_info_2d,
            dyn_resource_allocator,
            per_view_descriptor_sets_3d,
            descriptor_set_2d,
            ExtractedDebugData {
                line_lists_2d,
                line_lists_3d,
            },
        ))
    }

    fn feature_debug_name(&self) -> &'static str {
        Debug3dRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        Debug3dRenderFeature::feature_index()
    }
}
