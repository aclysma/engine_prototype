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

mod editable_handle;
pub use editable_handle::EditableHandle;

mod mesh_component;
pub use mesh_component::MeshComponent;
pub use mesh_component::MeshComponentDef;

mod point_light_component;
pub use point_light_component::PointLightComponent;

// #[derive(Copy, Clone)]
// pub struct PositionComponent {
//     pub position: Vec3,
// }

#[derive(Clone)]
pub struct DirectionalLightComponent {
    pub direction: glam::Vec3,
    pub color: glam::Vec3,
    pub intensity: f32,
}

#[derive(Clone)]
pub struct SpotLightComponent {
    pub direction: glam::Vec3,
    pub color: glam::Vec3,
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
