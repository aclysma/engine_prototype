use atelier_assets::loader::handle::Handle;
use crate::assets::gltf::MeshAsset;
use type_uuid::*;
use serde::{Serialize, Deserialize};
use serde_diff::{SerdeDiff};
use minimum::editor::EditorSelectableTransformed;
use legion::storage::{Archetype, Components, ComponentWriter};

use imgui_inspect_derive::Inspect;
use legion::{Entity, Resources, World, EntityStore};
use minimum::resources::editor::OpenedPrefabState;
use minimum::components::{TransformComponentDef};
use ncollide3d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide3d::world::CollisionWorld;
use minimum::resources::AssetResource;
use std::ops::{Range};
use legion_prefab::SpawnFrom;
use crate::components::EditableHandle;
use ncollide3d::shape::Cuboid;
use minimum::math::na_convert::vec3_glam_to_glm;

#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Default, Inspect)]
#[uuid = "46b6a84c-f224-48ac-a56d-46971bcaf7f1"]
pub struct MeshComponentDef {
    pub mesh: Option<EditableHandle<MeshAsset>>,
}

legion_prefab::register_component_type!(MeshComponentDef);

pub struct MeshComponent {
    //pub mesh_handle: MeshRenderNodeHandle,
    //pub visibility_handle: DynamicAabbVisibilityNodeHandle,
    pub mesh: Option<Handle<MeshAsset>>,
}

impl EditorSelectableTransformed<MeshComponent> for MeshComponentDef {
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        resources: &Resources,
        _opened_prefab: &OpenedPrefabState,
        prefab_world: &World,
        prefab_entity: Entity,
        _transformed_world: &World,
        transformed_entity: Entity,
        _transformed_component: &MeshComponent,
    ) {
        if let Some(mesh) = &self.mesh {
            let asset_resource = resources.get::<AssetResource>().unwrap();
            if let Some(mesh) = mesh.asset(asset_resource.storage()) {
                let bounding_aabb = &mesh.inner.asset.bounding_aabb;

                use ncollide3d::shape::ShapeHandle;
                if let Some(transform) = prefab_world
                    .entry_ref(prefab_entity)
                    .unwrap()
                    .get_component::<TransformComponentDef>()
                    .ok()
                {
                    let x = bounding_aabb.max.x() - bounding_aabb.min.x();
                    let y = bounding_aabb.max.y() - bounding_aabb.min.y();
                    let z = bounding_aabb.max.z() - bounding_aabb.min.z();
                    let mut half_extents = glam::Vec3::new(x, y, z) / 2.0;

                    let x = bounding_aabb.max.x() + bounding_aabb.min.x();
                    let y = bounding_aabb.max.y() + bounding_aabb.min.y();
                    let z = bounding_aabb.max.z() + bounding_aabb.min.z();
                    let center = glam::Vec3::new(x, y, z) / 2.0;

                    half_extents *= transform.scale();
                    let rotation = transform.rotation_quat();

                    half_extents.set_x(half_extents.x().abs().max(0.001));
                    half_extents.set_y(half_extents.y().abs().max(0.001));
                    half_extents.set_z(half_extents.z().abs().max(0.001));

                    let center = transform.position() + (center * transform.scale());

                    let shape_handle =
                        ShapeHandle::new(Cuboid::new(vec3_glam_to_glm(half_extents)));
                    let rotation = nalgebra::Quaternion::new(
                        rotation.w(),
                        rotation.x(),
                        rotation.y(),
                        rotation.z(),
                    );
                    let rotation = nalgebra::UnitQuaternion::from_quaternion(rotation);
                    collision_world.add(
                        ncollide3d::math::Isometry::from_parts(
                            nalgebra::Translation::from(vec3_glam_to_glm(center)),
                            rotation,
                        ),
                        shape_handle,
                        CollisionGroups::new(),
                        GeometricQueryType::Proximity(0.001),
                        transformed_entity,
                    );
                }
            }
        }
    }
}

impl SpawnFrom<MeshComponentDef> for MeshComponent {
    fn spawn_from(
        _resources: &Resources,
        src_entity_range: Range<usize>,
        src_arch: &Archetype,
        src_components: &Components,
        dst: &mut ComponentWriter<Self>,
        push_fn: fn(&mut ComponentWriter<Self>, Self),
    ) {
        // let mesh_render_nodes = resources.get::<MeshRenderNodeSet>().unwrap();
        // let dynamic_visibility_node_set =
        //     resources.get::<DynamicVisibilityNodeSet>().unwrap();
        let mesh_component_defs = legion_transaction::get_component_slice_from_archetype::<
            MeshComponentDef,
        >(src_components, src_arch, src_entity_range)
        .unwrap();

        for mesh_component_def in mesh_component_defs {
            // let mesh_render_node_handle = mesh_render_nodes.register_mesh(MeshRenderNode {
            //     entity: *dst_entity
            // });
            //
            // let visibility_node_handle = dynamic_visibility_node_set.register_dynamic_aabb(DynamicAabbVisibilityNode {
            //     handle: mesh_render_node_handle.into(),
            //     // aabb bounds
            // });

            let mesh_handle = mesh_component_def.mesh.as_ref().map(|x| x.handle.clone());
            let mesh_component = MeshComponent {
                //mesh_handle: mesh_render_node_handle,
                //visibility_handle: visibility_node_handle,
                mesh: mesh_handle, //delete_body_tx: physics.delete_body_tx().clone(),
            };

            (push_fn)(dst, mesh_component)

            //*into = std::mem::MaybeUninit::new()
        }
    }
}
