use crate::features::mesh::{
    ExtractedFrameNodeMeshData, MeshRenderNodeSet, MeshRenderFeature, MeshRenderNode, MeshDrawCall,
    MeshPerObjectShaderParam, ExtractedViewNodeMeshData, MeshPerViewShaderParam,
};
use crate::components::{
    PointLightComponent, SpotLightComponent, DirectionalLightComponent,
};
use crate::render_contexts::{RenderJobExtractContext, RenderJobWriteContext, RenderJobPrepareContext};
use renderer::nodes::{
    DefaultExtractJobImpl, FramePacket, RenderView, PerViewNode, PrepareJob, DefaultPrepareJob,
    RenderFeatureIndex, RenderFeature, PerFrameNode,
};
use renderer::base::slab::RawSlabKey;
use crate::features::mesh::prepare::MeshPrepareJobImpl;
use renderer::assets::resources::{PipelineSwapchainInfo, DescriptorSetAllocatorRef};
use atelier_assets::loader::handle::Handle;
use renderer::assets::resources::DescriptorSetArc;
use legion::*;
use crate::components::MeshComponent;
use crate::game_resource_manager::GameResourceManager;
use renderer::assets::MaterialAsset;
use minimum::components::{TransformComponent};

pub struct MeshExtractJobImpl {
    descriptor_set_allocator: DescriptorSetAllocatorRef,
    pipeline_info: PipelineSwapchainInfo,
    mesh_material: Handle<MaterialAsset>,
    descriptor_sets_per_view: Vec<DescriptorSetArc>,
    extracted_frame_node_mesh_data: Vec<Option<ExtractedFrameNodeMeshData>>,
    extracted_view_node_mesh_data: Vec<Vec<Option<ExtractedViewNodeMeshData>>>,
}

impl MeshExtractJobImpl {
    pub fn new(
        descriptor_set_allocator: DescriptorSetAllocatorRef,
        pipeline_info: PipelineSwapchainInfo,
        mesh_material: &Handle<MaterialAsset>,
    ) -> Self {
        MeshExtractJobImpl {
            descriptor_set_allocator,
            pipeline_info,
            mesh_material: mesh_material.clone(),
            descriptor_sets_per_view: Default::default(),
            extracted_frame_node_mesh_data: Default::default(),
            extracted_view_node_mesh_data: Default::default(),
        }
    }
}

impl DefaultExtractJobImpl<RenderJobExtractContext, RenderJobPrepareContext, RenderJobWriteContext>
    for MeshExtractJobImpl
{
    fn extract_begin(
        &mut self,
        _extract_context: &RenderJobExtractContext,
        frame_packet: &FramePacket,
        views: &[&RenderView],
    ) {
        self.extracted_frame_node_mesh_data
            .reserve(frame_packet.frame_node_count(self.feature_index()) as usize);

        self.extracted_view_node_mesh_data.reserve(views.len());
        for view in views {
            self.extracted_view_node_mesh_data.push(Vec::with_capacity(
                frame_packet.view_node_count(view, self.feature_index()) as usize,
            ));
        }
    }

    fn extract_frame_node(
        &mut self,
        extract_context: &RenderJobExtractContext,
        frame_node: PerFrameNode,
        _frame_node_index: u32,
    ) {
        let render_node_index = frame_node.render_node_index();
        let render_node_handle = RawSlabKey::<MeshRenderNode>::new(render_node_index);

        let mesh_nodes = extract_context
            .resources
            .get::<MeshRenderNodeSet>()
            .unwrap();
        let mesh_render_node = mesh_nodes.meshes.get(render_node_handle).unwrap();

        //TODO: Do this with queries? Probably requires moving the mesh node system data into ECS

        let entity = extract_context
            .world
            .entry_ref(mesh_render_node.entity)
            .unwrap();

        let transform_component = entity
            .get_component::<TransformComponent>()
            .ok();
        let mesh_component = entity
            .get_component::<MeshComponent>()
            .ok();

        let game_resource_manager = extract_context
            .resources
            .get::<GameResourceManager>();

        if transform_component.is_none() || mesh_component.is_none() || game_resource_manager.is_none() {
            self.extracted_frame_node_mesh_data.push(None);
            return;
        }
        let transform_component = transform_component.unwrap();
        let mesh_component = mesh_component.unwrap();
        let game_resource_manager = game_resource_manager.unwrap();

        let mesh_info = mesh_component.mesh.as_ref().and_then(|mesh_asset_handle| game_resource_manager.get_mesh_info(&mesh_asset_handle));
        if mesh_info.is_none() {
            self.extracted_frame_node_mesh_data.push(None);
            return;
        }
        let mesh_info = mesh_info.unwrap();

        let draw_calls: Vec<_> = mesh_info
            .mesh_asset
            .mesh_parts
            .iter()
            .map(|mesh_part| {
                let material_instance_info = extract_context
                    .resource_manager
                    .get_material_instance_info(&mesh_part.material_instance);
                let per_material_descriptor = material_instance_info.descriptor_sets[0][1].clone();
                MeshDrawCall {
                    vertex_buffer_offset_in_bytes: mesh_part.vertex_buffer_offset_in_bytes,
                    vertex_buffer_size_in_bytes: mesh_part.vertex_buffer_size_in_bytes,
                    index_buffer_offset_in_bytes: mesh_part.index_buffer_offset_in_bytes,
                    index_buffer_size_in_bytes: mesh_part.index_buffer_size_in_bytes,
                    per_material_descriptor,
                }
            })
            .collect();

        let world_transform = transform_component.transform();

        self.extracted_frame_node_mesh_data
            .push(Some(ExtractedFrameNodeMeshData {
                world_transform,
                vertex_buffer: mesh_info.vertex_buffer.clone(),
                index_buffer: mesh_info.index_buffer.clone(),
                draw_calls,
            }));
    }

    fn extract_view_node(
        &mut self,
        extract_context: &RenderJobExtractContext,
        view: &RenderView,
        view_node: PerViewNode,
        _view_node_index: u32,
    ) {
        let frame_node_data =
            &self.extracted_frame_node_mesh_data[view_node.frame_node_index() as usize];
        if frame_node_data.is_none() {
            self.extracted_view_node_mesh_data[view.view_index() as usize].push(None);
            return;
        }
        let frame_node_data = frame_node_data.as_ref().unwrap();

        let model_view = view.view_matrix() * frame_node_data.world_transform;
        let model_view_proj = view.projection_matrix() * model_view;

        let per_object_param = MeshPerObjectShaderParam {
            model_view,
            model_view_proj,
        };

        let layout =
            extract_context
                .resource_manager
                .get_descriptor_set_info(&self.mesh_material, 0, 2);
        let mut descriptor_set = self
            .descriptor_set_allocator
            .create_dyn_descriptor_set_uninitialized(&layout.descriptor_set_layout)
            .unwrap();
        descriptor_set.set_buffer_data(0, &per_object_param);
        descriptor_set
            .flush(&mut self.descriptor_set_allocator)
            .unwrap();

        self.extracted_view_node_mesh_data[view.view_index() as usize].push(Some(
            ExtractedViewNodeMeshData {
                per_instance_descriptor: descriptor_set.descriptor_set().clone(),
            },
        ))
    }

    fn extract_view_finalize(
        &mut self,
        extract_context: &RenderJobExtractContext,
        view: &RenderView,
    ) {
        let mut per_view_data = MeshPerViewShaderParam::default();

        let mut query = <Read<DirectionalLightComponent>>::query();
        for light in query.iter(extract_context.world) {
            let light_count = per_view_data.directional_light_count as usize;
            if light_count > per_view_data.directional_lights.len() {
                break;
            }

            let light_from = glam::Vec3::new(0.0, 0.0, 0.0);
            let light_from_vs = (view.view_matrix() * light_from.extend(1.0)).truncate();
            let light_to = *light.direction;
            let light_to_vs = (view.view_matrix() * light_to.extend(1.0)).truncate();

            let light_direction = (light_to - light_from).normalize();
            let light_direction_vs = (light_to_vs - light_from_vs).normalize();

            let out = &mut per_view_data.directional_lights[light_count];
            out.direction_ws = light_direction.into();
            out.direction_vs = light_direction_vs.into();
            out.color = light.color.extend(1.0);
            out.intensity = light.intensity;

            per_view_data.directional_light_count += 1;
        }

        let mut query = <(Read<TransformComponent>, Read<PointLightComponent>)>::query();
        for (transform, light) in query.iter(extract_context.world) {
            let light_count = per_view_data.point_light_count as usize;
            if light_count > per_view_data.point_lights.len() {
                break;
            }

            let out = &mut per_view_data.point_lights[light_count];
            out.position_ws = transform.position();
            out.position_vs = (view.view_matrix() * transform.position().extend(1.0)).truncate();
            out.color = light.color.extend(1.0);
            out.range = light.range;
            out.intensity = light.intensity * transform.uniform_scale().abs();

            per_view_data.point_light_count += 1;
        }

        let mut query = <(Read<TransformComponent>, Read<SpotLightComponent>)>::query();
        for (transform, light) in query.iter(extract_context.world) {
            let light_count = per_view_data.spot_light_count as usize;
            if light_count > per_view_data.spot_lights.len() {
                break;
            }

            let light_from = transform.position();
            let light_from_vs = (view.view_matrix().transform_point3(light_from));
            let light_to = transform.position() + *light.direction;
            let light_to_vs = (view.view_matrix().transform_point3(light_to));

            let light_direction = (light_to - light_from).normalize();
            let light_direction_vs = (light_to_vs - light_from_vs).normalize();

            let out = &mut per_view_data.spot_lights[light_count];
            out.position_ws = light_from.into();
            out.position_vs = light_from_vs.into();
            out.direction_ws = light_direction.into();
            out.direction_vs = light_direction_vs.into();
            out.spotlight_half_angle = light.spotlight_half_angle;
            out.color = light.color.extend(1.0);
            out.range = light.range;
            out.intensity = light.intensity * transform.uniform_scale().abs();

            per_view_data.spot_light_count += 1;
        }

        //TODO: We should probably set these up per view (so we can pick the best lights based on
        // the view)
        let layout =
            extract_context
                .resource_manager
                .get_descriptor_set_info(&self.mesh_material, 0, 0);
        let mut descriptor_set = self
            .descriptor_set_allocator
            .create_dyn_descriptor_set_uninitialized(&layout.descriptor_set_layout)
            .unwrap();
        descriptor_set.set_buffer_data(0, &per_view_data);
        descriptor_set
            .flush(&mut self.descriptor_set_allocator)
            .unwrap();

        self.descriptor_sets_per_view
            .push(descriptor_set.descriptor_set().clone());
    }

    fn extract_frame_finalize(
        self,
        _extract_context: &RenderJobExtractContext,
    ) -> Box<dyn PrepareJob<RenderJobPrepareContext, RenderJobWriteContext>> {
        let prepare_impl = MeshPrepareJobImpl::new(
            self.pipeline_info,
            self.descriptor_sets_per_view,
            self.extracted_frame_node_mesh_data,
            self.extracted_view_node_mesh_data,
        );

        Box::new(DefaultPrepareJob::new(prepare_impl))
    }

    fn feature_debug_name(&self) -> &'static str {
        MeshRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        MeshRenderFeature::feature_index()
    }
}
