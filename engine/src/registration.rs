use minimum::resources::*;
use minimum::components::*;

use minimum::editor::EditorSelectRegistry;
use minimum::editor::EditorSelectRegistryBuilder;
use minimum::editor::EditorInspectRegistry;
use minimum::editor::EditorInspectRegistryBuilder;

use minimum::ComponentRegistry;
use minimum::resources::editor::Keybinds;
use crate::components::MeshComponentDef;

/// Create the asset manager that has all the required types registered
pub fn create_asset_resource() -> AssetResource {
    let mut asset_manager = AssetResource::default();
    asset_manager.add_storage::<minimum::pipeline::PrefabAsset>();
    asset_manager
}

pub fn create_component_registry() -> ComponentRegistry {
    minimum::ComponentRegistryBuilder::new()
        .auto_register_components()
        // .add_spawn_mapping_into::<DrawSkiaCircleComponentDef, DrawSkiaCircleComponent>()
        // .add_spawn_mapping_into::<DrawSkiaBoxComponentDef, DrawSkiaBoxComponent>()
        // .add_spawn_mapping::<RigidBodyBallComponentDef, RigidBodyComponent>()
        // .add_spawn_mapping::<RigidBodyBoxComponentDef, RigidBodyComponent>()
        .build()
}

pub fn create_editor_selection_registry() -> EditorSelectRegistry {
    EditorSelectRegistryBuilder::new()
        // .register::<DrawSkiaBoxComponent>()
        // .register::<DrawSkiaCircleComponent>()
        // .register_transformed::<RigidBodyBoxComponentDef, RigidBodyComponent>()
        // .register_transformed::<RigidBodyBallComponentDef, RigidBodyComponent>()
        .build()
}

pub fn create_editor_inspector_registry() -> EditorInspectRegistry {
    EditorInspectRegistryBuilder::default()
        // .register::<DrawSkiaCircleComponentDef>()
        // .register::<DrawSkiaBoxComponentDef>()
        .register::<PositionComponent>()
        .register::<UniformScaleComponent>()
        .register::<NonUniformScaleComponent>()
        .register::<Rotation2DComponent>()
        .register::<MeshComponentDef>()
        // .register::<RigidBodyBallComponentDef>()
        // .register::<RigidBodyBoxComponentDef>()
        .build()
}

pub fn create_editor_keybinds() -> Keybinds {
    use minimum_sdl2::input::Sdl2KeyboardKey;
    use sdl2::keyboard::Keycode;
    Keybinds {
        selection_add: Sdl2KeyboardKey::new(Keycode::LShift).into(),
        selection_subtract: Sdl2KeyboardKey::new(Keycode::LAlt).into(),
        selection_toggle: Sdl2KeyboardKey::new(Keycode::LCtrl).into(),
        tool_translate: Sdl2KeyboardKey::new(Keycode::Num1).into(),
        tool_scale: Sdl2KeyboardKey::new(Keycode::Num2).into(),
        tool_rotate: Sdl2KeyboardKey::new(Keycode::Num3).into(),
        action_quit: Sdl2KeyboardKey::new(Keycode::Escape).into(),
        action_toggle_editor_pause: Sdl2KeyboardKey::new(Keycode::Space).into(),
    }
}
