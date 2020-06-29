// There is "dead" example code in this crate
#![allow(dead_code)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub use renderer;
pub use minimum;

use sdl2::event::Event;

use sdl2::mouse::MouseState;

use legion::prelude::*;

use minimum::resources::*;

mod systems;

mod registration;

use minimum_sdl2::imgui::Sdl2ImguiManager;

use renderer_shell_vulkan_sdl2::Sdl2Window;
use crate::components::{
    PositionComponent, PointLightComponent, SpotLightComponent, DirectionalLightComponent,
};

use crate::game_renderer::GameRenderer;
use crate::features::debug3d::DebugDraw3DResource;
use minimum::resources::editor::{
    EditorInspectRegistryResource, EditorMode, EditorStateResource, EditorSelectionResource,
    EditorSettingsResource, EditorDrawResource,
};
use crate::systems::{ScheduleCriteria, ScheduleManager};
use fnv::FnvHashMap;

mod asset_loader;
mod assets;
mod features;
mod game_renderer;
mod init;
mod test_scene;
mod game_resource_manager;
mod components;
mod game_asset_lookup;
mod renderpass;
mod phases;
mod render_contexts;

struct ImGuiInspectTest {
    mat4: minimum::math::Mat4,
}

pub fn run() {
    let mut resources = Resources::default();
    resources.insert(registration::create_asset_resource());
    resources.insert(AppControlResource::new());
    resources.insert(TimeResource::new());
    resources.insert(InputResource::new());
    resources.insert(EditorStateResource::new());
    resources.insert(DebugDrawResource::new());
    resources.insert(EditorDrawResource::new());
    resources.insert(EditorSettingsResource::new(
        registration::create_editor_keybinds(),
    ));
    resources.insert(EditorSelectionResource::new(
        registration::create_editor_selection_registry(),
    ));
    resources.insert(EditorInspectRegistryResource::new(
        registration::create_editor_inspector_registry(),
    ));
    resources.insert(ComponentRegistryResource::new(
        registration::create_component_registry(),
    ));

    let sdl2_systems = init::sdl2_init();
    let window_size = sdl2_systems.window.drawable_size();
    let viewport_size = ViewportSize::new(window_size.0, window_size.1);

    let camera = CameraResource::new(glam::Vec2::new(0.0, 1.0), 10.0);
    let viewport = ViewportResource::new(viewport_size, camera.position, camera.x_half_extents);

    resources.insert(camera);
    resources.insert(viewport);

    // This will register more rendering-specific asset types
    init::rendering_init(&mut resources, &sdl2_systems.window);

    log::info!("Starting window event loop");
    let mut event_pump = sdl2_systems
        .context
        .event_pump()
        .expect("Could not create sdl event pump");

    let universe = Universe::new();
    let mut world = universe.create_world();
    resources.insert(UniverseResource::new(universe));

    test_scene::populate_test_sprite_entities(&mut resources, &mut world);
    test_scene::populate_test_mesh_entities(&mut resources, &mut world);
    test_scene::populate_test_lights(&mut resources, &mut world);

    let mut schedule_manager = ScheduleManager::new();

    //let mut print_time_event = minimum::util::PeriodicEvent::default();

    let sdl2_imgui = resources.get::<Sdl2ImguiManager>().unwrap().clone();

    'running: loop {
        let t0 = std::time::Instant::now();

        for event in event_pump.poll_iter() {
            log::info!("SDL2 Event: {:?}", event);

            let mut input_resource = resources.get_mut::<InputResource>().unwrap();

            sdl2_imgui.handle_event(&event);

            if !sdl2_imgui.ignore_event(&event) {
                let viewport = resources.get_mut::<ViewportResource>().unwrap();
                minimum_sdl2::input::handle_sdl2_event(
                    &event,
                    input_resource.input_state_mut(),
                    &*viewport,
                );

                match event {
                    //
                    // Halt if the user requests to close the window
                    //
                    Event::Quit { .. } => break 'running,

                    _ => {}
                }
            }
        }

        //
        // Notify imgui of frame begin
        //
        sdl2_imgui.begin_frame(&sdl2_systems.window, &MouseState::new(&event_pump));

        schedule_manager.update(&mut world, &mut resources);

        //
        // Close imgui input for this frame and render the results to memory
        //
        sdl2_imgui.render(&sdl2_systems.window);

        let t1 = std::time::Instant::now();
        log::trace!(
            "[main] simulation took {} ms",
            (t1 - t0).as_secs_f32() * 1000.0
        );

        //
        // Redraw
        //
        {
            let window = Sdl2Window::new(&sdl2_systems.window);
            let game_renderer = resources.get::<GameRenderer>().unwrap();
            game_renderer
                .begin_render(&resources, &world, &window)
                .unwrap();
        }

        if resources
            .get::<AppControlResource>()
            .unwrap()
            .should_terminate_process()
        {
            break;
        }

        //let t2 = std::time::Instant::now();
        //log::info!("main thread took {} ms", (t2 - t0).as_secs_f32() * 1000.0);
    }

    // Remove the asset resource because we have asset storages that reference resources
    resources.remove::<AssetResource>();

    // Tear down rendering
    init::rendering_destroy(&mut resources);

    // Drop world before resources as physics components may point back at resources
    std::mem::drop(world);
    std::mem::drop(resources);
}

//TODO:
// * Init these resources
// (X - temp) CameraResource - Cameras should be components, not resources
// ( ) Sdl2WindowResource - Unnecessary
// (X - temp) ViewportResource - Populate with camera data (YES INIT IT PROBABLY?)
// (X - temp) DebugDrawResource - Rename to debugdraw 2d?
// (X - temp) EditorDrawResource - Implemented in 2d
// (X) EditorSelectionResource - Should I even bother with these? TEMPORARY
// (X) EditorInspectRegistryResource - YES, INIT IT
// (X) ComponentRegistryResource - YES, INIT IT
// (X) Sdl2ImguiManagerResource- PROBABLY
// (X) ImguiResource - PROBABLY
// (X) AppControlResource - YES, but should this just be a message channel?
// (X) TimeResource - DONE
// (X) InputResource _ YES, INIT IT
// (-) CanvasDrawResource - no
// (X) UniverseResource - YES, INIT IT
// (-) Sdl2WindowResource - no
// (X) EditorStateResource - PROBABALY FOR NOW
// (X) EditorSettingsResource - PROBABLY FOR NOW
//
// ( ) Tick existing systems
// ( ) Port current update code into systems
// ( ) Check debug draw/editor draw are ok
// ( ) 3D debug draw?
// ( ) Load a scene
