use crate::features::mesh::MeshRenderNodeHandle;
use renderer::visibility::DynamicAabbVisibilityNodeHandle;
use atelier_assets::loader::handle::Handle;
use crate::assets::gltf::MeshAsset;
use glam::f32::Vec3;
use crate::features::sprite::SpriteRenderNodeHandle;
use renderer::assets::ImageAsset;
use type_uuid::*;
use serde::{Serialize, Deserialize};
use serde_diff::SerdeDiff;

use imgui_inspect_derive::Inspect;

#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Inspect, Default)]
#[uuid = "46b6a84c-f224-48ac-a56d-46971bcaf7f1"]
pub struct MeshComponentDef {
    #[serde_diff(opaque)]
    #[inspect(skip)]
    pub mesh: Option<Handle<MeshAsset>>
}

legion_prefab::register_component_type!(MeshComponentDef);

pub struct MeshComponent {
    pub mesh_handle: MeshRenderNodeHandle,
    pub visibility_handle: DynamicAabbVisibilityNodeHandle,
    pub mesh: Handle<MeshAsset>,
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
    //pub texture_material: ResourceArc<>
}
