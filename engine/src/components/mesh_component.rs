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
use minimum::components::{UniformScaleComponent, NonUniformScaleComponent, Rotation2DComponent, PositionComponent};
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


pub fn vec3_glam_to_glm(value: glam::Vec3) -> nalgebra_glm::Vec3 {
    nalgebra_glm::Vec3::new(value.x(), value.y(), value.z())
}


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
                if let Some(position) = prefab_world.get_component::<PositionComponent>(prefab_entity) {


                    //let mut half_extents = *self.half_extents;

                    // if let Some(uniform_scale) =
                    // prefab_world.get_component::<UniformScaleComponent>(prefab_entity)
                    // {
                    //     half_extents *= uniform_scale.uniform_scale;
                    // }
                    //
                    // if let Some(non_uniform_scale) =
                    // prefab_world.get_component::<NonUniformScaleComponent>(prefab_entity)
                    // {
                    //     half_extents *= *non_uniform_scale.non_uniform_scale;
                    // }

                    // let mut rotation = 0.0;
                    // if let Some(rotation_component) =
                    // prefab_world.get_component::<Rotation2DComponent>(prefab_entity)
                    // {
                    //     rotation = rotation_component.rotation;
                    // }


                    // let shape_handle = ShapeHandle::new(Ball::new(
                    //     bounding_sphere.radius
                    // ));
                    // let rotation = nalgebra::UnitQuaternion::identity();
                    // collision_world.add(
                    //     ncollide3d::math::Isometry::from_parts(
                    //         nalgebra::Translation::from(vec3_glam_to_glm(*position.position + bounding_sphere.center)),
                    //         rotation,
                    //     ),
                    //     shape_handle,
                    //     CollisionGroups::new(),
                    //     GeometricQueryType::Proximity(0.001),
                    //     transformed_entity,
                    // );

                    let x = bounding_aabb.max.x() - bounding_aabb.min.x();
                    let y = bounding_aabb.max.y() - bounding_aabb.min.y();
                    let z = bounding_aabb.max.z() - bounding_aabb.min.z();
                    let half_extents = glam::Vec3::new(x, y, z) / 2.0;

                    let x = bounding_aabb.max.x() + bounding_aabb.min.x();
                    let y = bounding_aabb.max.y() + bounding_aabb.min.y();
                    let z = bounding_aabb.max.z() + bounding_aabb.min.z();
                    let center = glam::Vec3::new(x, y, z) / 2.0;

                    let shape_handle = ShapeHandle::new(Cuboid::new(
                        ncollide3d::math::Vector::from(vec3_glam_to_glm(half_extents))
                    ));
                    let rotation = nalgebra::UnitQuaternion::identity();
                    collision_world.add(
                        ncollide3d::math::Isometry::from_parts(
                            nalgebra::Translation::from(vec3_glam_to_glm(*position.position + center)),
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

// fn transform_shape_to_rigid_body(
//     physics: &mut PhysicsResource,
//     into: &mut std::mem::MaybeUninit<RigidBodyComponent>,
//     src_position: Option<&PositionComponent>,
//     _src_rotation: Option<&Rotation2DComponent>,
//     shape_handle: ShapeHandle<f32>,
//     is_static: bool,
// ) {
//     let position = if let Some(position) = src_position {
//         position.position
//     } else {
//         Vec3::zero()
//     };
//
//     let mut collider_offset = Vec3::zero();
//
//     // Build the rigid body.
//     let rigid_body_handle = if is_static {
//         *collider_offset += *position;
//         physics.bodies.insert(nphysics2d::object::Ground::new())
//     } else {
//         physics.bodies.insert(
//             nphysics2d::object::RigidBodyDesc::new()
//                 .translation(vec2_glam_to_glm(position.xy().into()))
//                 .build(),
//         )
//     };
//
//     // Build the collider.
//     let collider = nphysics2d::object::ColliderDesc::new(shape_handle.clone())
//         .density(1.0)
//         .translation(vec2_glam_to_glm(*collider_offset.xy()))
//         .build(nphysics2d::object::BodyPartHandle(rigid_body_handle, 0));
//
//     // Insert the collider to the body set.
//     physics.colliders.insert(collider);
//
//     *into = std::mem::MaybeUninit::new(RigidBodyComponent {
//         handle: rigid_body_handle,
//         delete_body_tx: physics.delete_body_tx().clone(),
//     })
// }

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
        //let mut physics = resources.get_mut::<PhysicsResource>().unwrap();

        // let position_components = iter_components_in_storage::<PositionComponent>(
        //     src_component_storage,
        //     src_component_storage_indexes.clone(),
        // );
        //
        // let uniform_scale_components = iter_components_in_storage::<UniformScaleComponent>(
        //     src_component_storage,
        //     src_component_storage_indexes.clone(),
        // );
        //
        // let rotation_components = iter_components_in_storage::<Rotation2DComponent>(
        //     src_component_storage,
        //     src_component_storage_indexes,
        // );


        let mut mesh_render_nodes = resources.get_mut::<MeshRenderNodeSet>().unwrap();
        let mut dynamic_visibility_node_set =
            resources.get_mut::<DynamicVisibilityNodeSet>().unwrap();

        for (/*src_position, src_uniform_scale, src_rotation,*/ from, into, dst_entity) in izip!(
            // position_components,
            // uniform_scale_components,
            // rotation_components,
            from,
            into,
            dst_entities
        ) {
            // mesh_render_nodes.register_mesh_with_handle(|mesh_handle| {
            //     let aabb_info = DynamicAabbVisibilityNode {
            //         handle: mesh_handle.into(),
            //         // aabb bounds
            //     };
            //
            //     // User calls functions to register visibility objects
            //     // - This is a retained API because presumably we don't want to rebuild spatial structures every frame
            //     let visibility_handle = dynamic_visibility_node_set.register_dynamic_aabb(aabb_info);
            //
            //     let position_component = PositionComponent { position };
            //     let mesh_component = MeshComponent {
            //         mesh_handle,
            //         visibility_handle,
            //         mesh: mesh.clone(),
            //     };
            //
            //     let entity = world.insert((), vec![(position_component, mesh_component)])[0];
            //
            //     world.get_component::<PositionComponent>(entity).unwrap();
            //
            //     MeshRenderNode {
            //         entity, // sprite asset
            //     }
            // });

            let mesh_render_node_handle = mesh_render_nodes.register_mesh(MeshRenderNode {
                entity: *dst_entity
            });

            let aabb_info = DynamicAabbVisibilityNode {
                handle: mesh_render_node_handle.into(),
                // aabb bounds
            };
            let visibility_node_handle = dynamic_visibility_node_set.register_dynamic_aabb(aabb_info);

            // pub struct MeshComponent {
            //     pub mesh_handle: MeshRenderNodeHandle,
            //     pub visibility_handle: DynamicAabbVisibilityNodeHandle,
            //     pub mesh: Handle<MeshAsset>,
            // }


            let mesh_handle = from.mesh.as_ref().map(|x| x.handle.clone());

            *into = std::mem::MaybeUninit::new(MeshComponent {
                mesh_handle: mesh_render_node_handle,
                visibility_handle: visibility_node_handle,
                mesh: mesh_handle
                //delete_body_tx: physics.delete_body_tx().clone(),
            })


            // let mut radius = from.radius;
            // if let Some(src_uniform_scale) = src_uniform_scale {
            //     radius *= src_uniform_scale.uniform_scale;
            // }

            // //TODO: Warn if radius is 0
            // let shape_handle = ShapeHandle::new(Ball::new(radius.max(0.01)));
            // transform_shape_to_rigid_body(
            //     &mut physics,
            //     into,
            //     src_position,
            //     src_rotation,
            //     shape_handle,
            //     from.is_static,
            // );
        }
    }
}
