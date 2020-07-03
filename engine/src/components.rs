use crate::features::mesh::MeshRenderNodeHandle;
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

use imgui_inspect_derive::Inspect;
use legion::prelude::{Entity, Resources, World, EntityStore};
use minimum::resources::editor::OpenedPrefabState;
use minimum::components::{UniformScaleComponent, NonUniformScaleComponent, Rotation2DComponent};
use ncollide3d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide3d::world::CollisionWorld;
use minimum::resources::AssetResource;
use std::marker::PhantomData;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectArgsStruct};
use std::ops::{Deref, DerefMut};

pub fn vec3_glam_to_glm(value: glam::Vec3) -> nalgebra_glm::Vec3 {
    nalgebra_glm::Vec3::new(value.x(), value.y(), value.z())
}

#[derive(Eq)]
pub struct EditableHandle<T: ?Sized> {
    handle: Handle<T>
}

impl<T: ?Sized> PartialEq for EditableHandle<T> {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.handle == other.handle
    }
}

impl<T: ?Sized> Clone for EditableHandle<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<T> std::fmt::Debug for EditableHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditableHandle")
            .field("handle", &self.handle)
            .finish()
    }
}

impl<T> Deref for EditableHandle<T> {
    type Target = Handle<T>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<T> DerefMut for EditableHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl<T> Serialize for EditableHandle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        self.handle.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for EditableHandle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let handle = <Handle<T> as Deserialize>::deserialize(deserializer)?;
        Ok(EditableHandle {
            handle
        })
    }
}

impl<T> SerdeDiff for EditableHandle<T> {
    fn diff<'a, S: serde::ser::SerializeSeq>(&self, ctx: &mut DiffContext<'a, S>, other: &Self) -> Result<bool, <S as serde::ser::SerializeSeq>::Error> {
        if self.handle != other.handle {
            ctx.save_value(other)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn apply<'de, A>(&mut self, seq: &mut A, ctx: &mut ApplyContext) -> Result<bool, <A as serde::de::SeqAccess<'de>>::Error> where
        A: serde::de::SeqAccess<'de> {
        ctx.read_value(seq, self)
    }
}

impl<T> imgui_inspect::InspectRenderDefault<EditableHandle<T>> for EditableHandle<T> {
    fn render(data: &[&EditableHandle<T>], label: &'static str, ui: &Ui, args: &InspectArgsDefault) {
        ui.text(imgui::im_str!("hi {:?}", data[0].handle));
    }

    fn render_mut(data: &mut [&mut EditableHandle<T>], label: &'static str, ui: &Ui, args: &InspectArgsDefault) -> bool {
        ui.text(imgui::im_str!("hi {:?}", data[0].handle));
        false
    }
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
    pub mesh: Handle<MeshAsset>,
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


                    let shape_handle = ShapeHandle::new(Ball::new(
                        bounding_sphere.radius
                    ));
                    let rotation = nalgebra::UnitQuaternion::identity();


                    collision_world.add(
                        ncollide3d::math::Isometry::from_parts(
                            nalgebra::Translation::from(vec3_glam_to_glm(position.position + bounding_sphere.center)),
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

#[derive(Copy, Clone)]
pub struct PositionComponent {
    pub position: Vec3,
}

#[derive(Clone)]
pub struct PointLightComponent {
    pub color: glam::Vec4,
    pub range: f32,
    pub intensity: f32,
}

#[derive(Clone)]
pub struct DirectionalLightComponent {
    pub direction: glam::Vec3,
    pub color: glam::Vec4,
    pub intensity: f32,
}

#[derive(Clone)]
pub struct SpotLightComponent {
    pub direction: glam::Vec3,
    pub color: glam::Vec4,
    pub spotlight_half_angle: f32,
    pub range: f32,
    pub intensity: f32,
}

#[derive(Clone)]
pub struct SpriteComponent {
    pub sprite_handle: SpriteRenderNodeHandle,
    pub visibility_handle: DynamicAabbVisibilityNodeHandle,
    pub alpha: f32,
    pub image: Handle<ImageAsset>,
}
