use crate::features::mesh::{MeshRenderNodeHandle, MeshRenderNodeSet, MeshRenderNode};
use renderer::visibility::DynamicAabbVisibilityNodeHandle;
use atelier_assets::loader::handle::Handle;
use crate::assets::gltf::MeshAsset;
use glam::f32::Vec3;
use crate::features::sprite::SpriteRenderNodeHandle;
use renderer::assets::ImageAsset;
use type_uuid::*;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde_diff::{SerdeDiff, DiffContext, ApplyContext};
use minimum::editor::EditorSelectableTransformed;
use legion::storage::ComponentStorage;
use legion::index::ComponentIndex;
use renderer::visibility::DynamicVisibilityNodeSet;
use renderer::visibility::DynamicAabbVisibilityNode;

use imgui_inspect_derive::Inspect;
use legion::prelude::{Entity, Resources, World, EntityStore};
use minimum::resources::editor::OpenedPrefabState;
use minimum::components::{TransformComponentDef};
use ncollide3d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide3d::world::CollisionWorld;
use minimum::resources::AssetResource;
use std::marker::PhantomData;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectArgsStruct};
use std::ops::{Deref, DerefMut, Range};
use legion_prefab::SpawnFrom;
use legion_transaction::iter_components_in_storage;
use crate::components::EditableHandle;
use ncollide3d::shape::Cuboid;
use minimum::math::na_convert::vec3_glam_to_glm;


#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Default, Inspect)]
#[uuid = "46b6a84c-f224-48ac-a56d-46971bcaf7f1"]
pub struct MeshComponentDef {
    pub mesh: Option<EditableHandle<MeshAsset>>
}

legion_prefab::register_component_type!(MeshComponentDef);

pub struct MeshComponent {
    pub mesh_handle: MeshRenderNodeHandle,
    pub visibility_handle: DynamicAabbVisibilityNodeHandle,
    pub mesh: Option<Handle<MeshAsset>>,
}

impl EditorSelectableTransformed<MeshComponent> for MeshComponentDef {
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        resources: &Resources,
        opened_prefab: &OpenedPrefabState,
        prefab_world: &World,
        prefab_entity: Entity,
        transformed_world: &World,
        transformed_entity: Entity,
        transformed_component: &MeshComponent
    ) {
        if let Some(mesh) = &self.mesh {
            let asset_resource = resources.get::<AssetResource>().unwrap();
            if let Some(mesh) = mesh.asset(asset_resource.storage()) {
                let bounding_sphere = &mesh.inner.asset.bounding_sphere;
                let bounding_aabb = &mesh.inner.asset.bounding_aabb;

                use ncollide3d::shape::ShapeHandle;
                use ncollide3d::shape::Ball;
                if let Some(transform) = prefab_world.get_component::<TransformComponentDef>(prefab_entity) {
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

                    let shape_handle = ShapeHandle::new(Cuboid::new(
                        ncollide3d::math::Vector::from(vec3_glam_to_glm(half_extents))
                    ));
                    let rotation = nalgebra::Quaternion::new(rotation.w(), rotation.x(), rotation.y(), rotation.z());
                    let rotation = nalgebra::UnitQuaternion::from_quaternion(rotation);
                    collision_world.add(
                        ncollide3d::math::Isometry::from_parts(
                            nalgebra::Translation::from(vec3_glam_to_glm( center)),
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
        _src_world: &World,
        src_component_storage: &ComponentStorage,
        src_component_storage_indexes: Range<ComponentIndex>,
        resources: &Resources,
        _src_entities: &[Entity],
        dst_entities: &[Entity],
        from: &[MeshComponentDef],
        into: &mut [std::mem::MaybeUninit<Self>],
    ) {
        let mut mesh_render_nodes = resources.get_mut::<MeshRenderNodeSet>().unwrap();
        let mut dynamic_visibility_node_set =
            resources.get_mut::<DynamicVisibilityNodeSet>().unwrap();

        for (from, into, dst_entity) in izip!(
            from,
            into,
            dst_entities
        ) {
            let mesh_render_node_handle = mesh_render_nodes.register_mesh(MeshRenderNode {
                entity: *dst_entity
            });

            let aabb_info = DynamicAabbVisibilityNode {
                handle: mesh_render_node_handle.into(),
                // aabb bounds
            };
            let visibility_node_handle = dynamic_visibility_node_set.register_dynamic_aabb(aabb_info);

            let mesh_handle = from.mesh.as_ref().map(|x| x.handle.clone());

            *into = std::mem::MaybeUninit::new(MeshComponent {
                mesh_handle: mesh_render_node_handle,
                visibility_handle: visibility_node_handle,
                mesh: mesh_handle
                //delete_body_tx: physics.delete_body_tx().clone(),
            })
        }
    }
}
