use type_uuid::TypeUuid;
use serde::{Serialize, Deserialize};
use serde_diff::SerdeDiff;
use imgui_inspect_derive::Inspect;
use minimum::math::Vec3;
use legion::{World, Entity, Resources, EntityStore};
use minimum::resources::editor::OpenedPrefabState;
use minimum::components::{TransformComponent};
use ncollide3d::shape::{ShapeHandle, Ball};
use ncollide3d::world::CollisionWorld;
use ncollide3d::pipeline::{GeometricQueryType, CollisionGroups};
use minimum::editor::EditorSelectable;
use minimum::math::na_convert::vec3_glam_to_glm;

#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Default, Inspect)]
#[uuid = "84c8de08-f5ea-48f5-8bbd-f56a30b6aecf"]
pub struct PointLightComponent {
    #[serde_diff(opaque)]
    pub color: Vec3,
    pub range: f32,
    pub intensity: f32,
}

legion_prefab::register_component_type!(PointLightComponent);

impl EditorSelectable for PointLightComponent {
    fn create_editor_selection_world(
        &self,
        collision_world: &mut CollisionWorld<f32, Entity>,
        _resources: &Resources,
        _opened_prefab: &OpenedPrefabState,
        prefab_world: &World,
        prefab_entity: Entity,
    ) {
        if let Some(transform) = prefab_world
            .entry_ref(prefab_entity)
            .unwrap()
            .get_component::<TransformComponent>()
            .ok()
        {
            let shape_handle = ShapeHandle::new(Ball::new(0.25));
            let rotation = nalgebra::UnitQuaternion::identity();
            collision_world.add(
                ncollide3d::math::Isometry::from_parts(
                    nalgebra::Translation::from(vec3_glam_to_glm(transform.position())),
                    rotation,
                ),
                shape_handle,
                CollisionGroups::new(),
                GeometricQueryType::Proximity(0.001),
                prefab_entity,
            );
        }
    }
}
