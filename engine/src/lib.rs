// There is "dead" example code in this crate
#![allow(dead_code)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

#[allow(unused_imports)]
#[macro_use]
extern crate itertools;

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
    PointLightComponent, SpotLightComponent, DirectionalLightComponent,
};

use crate::game_renderer::GameRenderer;
use minimum::resources::editor::{
    EditorInspectRegistryResource, EditorMode, EditorStateResource, EditorSelectionResource,
    EditorSettingsResource, EditorDraw3DResource,
};
use crate::systems::{ScheduleCriteria, ScheduleManager};
use fnv::FnvHashMap;
use atelier_assets::core as atelier_core;
use atelier_assets::core::asset_uuid;
use atelier_assets::loader::rpc_loader::RpcLoader;

mod asset_loader;
pub mod assets;
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

pub fn run(connect_string: String) {
    let mut resources = Resources::default();
    let loader = RpcLoader::new(connect_string).unwrap();
    resources.insert(registration::create_asset_resource(loader));
    resources.insert(AppControlResource::new());
    resources.insert(TimeResource::new());
    resources.insert(InputResource::new());
    resources.insert(EditorStateResource::new());
    resources.insert(DebugDraw3DResource::new());
    resources.insert(EditorDraw3DResource::new());
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
    //let viewport_size = ViewportSize::new(window_size.0, window_size.1);

    let camera = CameraResource::new(glam::Vec2::new(0.0, 1.0), 10.0);
    let mut viewport = ViewportResource::empty();
    viewport.set_viewport_size_in_pixels(glam::Vec2::new(window_size.0 as f32, window_size.1 as f32));
    viewport.set_screen_space_view(glam::Mat4::identity());
    viewport.set_world_space_view(glam::Mat4::identity(), glam::Mat4::identity(), glam::Vec3::zero());

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

    // test_scene::populate_test_sprite_entities(&mut resources, &mut world);
    //test_scene::populate_test_mesh_entities(&mut resources, &mut world);
    //test_scene::populate_test_lights(&mut resources, &mut world);

    let mut schedule_manager = ScheduleManager::new();

    //let mut print_time_event = minimum::util::PeriodicEvent::default();

    #[cfg(feature = "use_imgui")]
    let sdl2_imgui = resources.get::<Sdl2ImguiManager>().unwrap().clone();

    //EditorStateResource::open_prefab(&mut world, &resources, asset_uuid!("12b37b66-94f7-4fa6-abb3-4050619c3e11")).unwrap();
    EditorStateResource::open_prefab(&mut world, &resources, asset_uuid!("2aad7b4c-a323-415a-bea6-ae0f945446b9")).unwrap();

    'running: loop {
        let t0 = std::time::Instant::now();

        for event in event_pump.poll_iter() {
            //log::info!("SDL2 Event: {:?}", event);

            let mut input_resource = resources.get_mut::<InputResource>().unwrap();

            #[cfg(feature = "use_imgui")]
            sdl2_imgui.handle_event(&event);

            let mut ignore_event = false;

            #[cfg(feature = "use_imgui")]
            {
                ignore_event |= sdl2_imgui.ignore_event(&event);
            }

            if !ignore_event {
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
        #[cfg(feature = "use_imgui")]
        sdl2_imgui.begin_frame(&sdl2_systems.window, &MouseState::new(&event_pump));

        schedule_manager.update(&mut world, &mut resources);

        //
        // Close imgui input for this frame and render the results to memory
        //
        #[cfg(feature = "use_imgui")]
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
