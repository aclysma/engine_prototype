
use type_uuid::TypeUuid;
use serde::{Serialize, Deserialize};
use serde_diff::SerdeDiff;
use imgui_inspect_derive::Inspect;
use minimum::math::Vec4;

#[derive(TypeUuid, Serialize, Deserialize, SerdeDiff, Debug, PartialEq, Clone, Default, Inspect)]
#[uuid = "84c8de08-f5ea-48f5-8bbd-f56a30b6aecf"]
pub struct PointLightComponent {
    #[serde_diff(opaque)]
    pub color: Vec4,
    pub range: f32,
    pub intensity: f32,
}

legion_prefab::register_component_type!(PointLightComponent);
