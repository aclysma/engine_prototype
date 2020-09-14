use renderer::visibility::DynamicAabbVisibilityNodeHandle;
use atelier_assets::loader::handle::Handle;
use crate::features::sprite::SpriteRenderNodeHandle;
use renderer::assets::ImageAsset;

mod editable_handle;
pub use editable_handle::EditableHandle;

mod mesh_component;
pub use mesh_component::MeshComponent;
pub use mesh_component::MeshComponentDef;

mod point_light_component;
pub use point_light_component::PointLightComponent;

mod spot_light_component;
pub use spot_light_component::SpotLightComponent;

mod directional_light_component;
pub use directional_light_component::DirectionalLightComponent;

// #[derive(Copy, Clone)]
// pub struct PositionComponent {
//     pub position: Vec3,
// }

#[derive(Clone)]
pub struct SpriteComponent {
    pub render_node: SpriteRenderNodeHandle,
    pub visibility_node: DynamicAabbVisibilityNodeHandle,
    pub alpha: f32,
    pub image: Handle<ImageAsset>,
}
