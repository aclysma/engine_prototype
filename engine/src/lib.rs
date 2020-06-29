// There is "dead" example code in this crate
#![allow(dead_code)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub use renderer;
pub use minimum;

use sdl2::event::Event;

use sdl2::mouse::MouseState;
use sdl2::keyboard::Keycode;

use legion::prelude::*;

use minimum::resources::*;

mod systems;

mod registration;

use minimum_sdl2::imgui::Sdl2ImguiManager;

use renderer::vulkan::VkDeviceContext;
use renderer_shell_vulkan_sdl2::Sdl2Window;
use crate::components::{
    PositionComponent, PointLightComponent, SpotLightComponent, DirectionalLightComponent,
};

use renderer::assets::ResourceManager;
use crate::game_renderer::GameRenderer;
use crate::features::debug3d::DebugDraw3DResource;
use crate::game_resource_manager::GameResourceManager;
use minimum::resources::editor::EditorInspectRegistryResource;

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
    resources.insert(TimeState::new());
    resources.insert(registration::create_asset_resource());

    resources.insert(EditorInspectRegistryResource::new(
        registration::create_editor_inspector_registry(),
    ));
    resources.insert(ComponentRegistryResource::new(
        registration::create_component_registry(),
    ));

    let sdl2_systems = init::sdl2_init();

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

    let mut print_time_event = minimum::util::PeriodicEvent::default();

    'running: loop {
        let t0 = std::time::Instant::now();
        //
        // Update time
        //
        {
            resources.get_mut::<TimeState>().unwrap().update();
        }

        //
        // Print FPS
        //
        {
            let time_state = resources.get::<TimeState>().unwrap();
            if print_time_event.try_take_event(
                time_state.current_instant(),
                std::time::Duration::from_secs_f32(1.0),
            ) {
                log::info!("FPS: {}", time_state.updates_per_second());
                //renderer.dump_stats();
            }
        }

        //
        // Notify imgui of frame begin
        //
        {
            let imgui_manager = resources.get::<Sdl2ImguiManager>().unwrap();
            imgui_manager.begin_frame(&sdl2_systems.window, &MouseState::new(&event_pump));
        }

        //
        // Update assets
        //
        {
            let mut asset_resource = resources.get_mut::<AssetResource>().unwrap();
            asset_resource.update();
        }

        //
        // Update graphics resources
        //
        {
            let mut resource_manager = resources.get_mut::<ResourceManager>().unwrap();
            let mut game_resource_manager = resources.get_mut::<GameResourceManager>().unwrap();

            resource_manager.update_resources().unwrap();
            game_resource_manager
                .update_resources(&*resource_manager)
                .unwrap();
        }

        //
        // Process input
        //
        if !process_input(&resources, &mut event_pump) {
            break 'running;
        }

        add_light_debug_draw(&resources, &world);

        //
        // imgui debug draw,
        //
        {
            let imgui_manager = resources.get::<Sdl2ImguiManager>().unwrap();
            let time_state = resources.get::<TimeState>().unwrap();
            imgui_manager.with_ui(|ui| {
                ui.main_menu_bar(|| {
                    ui.text(imgui::im_str!(
                        "FPS: {:.1}",
                        time_state.updates_per_second_smoothed()
                    ));
                    ui.separator();
                    ui.text(imgui::im_str!("Frame: {}", time_state.update_count()));
                });

                //ui.window
            });
        }

        //
        // Close imgui input for this frame and render the results to memory
        //
        {
            let imgui_manager = resources.get::<Sdl2ImguiManager>().unwrap();
            imgui_manager.render(&sdl2_systems.window);
        }

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
// ( ) CameraResource - Cameras should be components, not resources
// ( ) Sdl2WindowResource - Unnecessary
// ( ) ViewportResource - Populate with camera data (YES INIT IT PROBABLY?)
// ( ) DebugDrawResource - Rename to debugdraw 2d?
// ( ) EditorDrawResource - Implemented in 2d
// ( ) EditorSelectionResource - Should I even bother with these?
// (X) EditorInspectRegistryResource - YES, INIT IT
// (X) ComponentRegistryResource - YES, INIT IT
// ( ) Sdl2ImguiManagerResource- PROBABLY
// ( ) ImguiResource - PROBABLY
// ( ) AppControlResource - YES, but should this just be a message channel?
// ( ) TimeResource - DONE
// ( ) InputResource _ YES, INIT IT
// ( ) CanvasDrawResource - no
// ( ) UniverseResource - YES, INIT IT
// ( ) Sdl2WindowResource - no
// ( ) EditorStateResource - PROBABALY FOR NOW
// ( ) EditorSettingsResource - PROBABLY FOR NOW
//
// * Tick existing systems
// * Port current update code into systems
// * Check debug draw/editor draw are ok
// * 3D debug draw?
// * Load a scene

/*
fn create_resources(
    universe: Universe,
    sdl2_window: &sdl2::video::Window,
    sdl2_imgui: &Sdl2ImguiManager,
) -> Resources {
    let mut resources = Resources::default();

    let asset_resource = registration::create_asset_manager();

    let physics_resource = PhysicsResource::new(glam::Vec2::unit_y() * GRAVITY);

    let camera_resource = CameraResource::new(
        glam::Vec2::new(0.0, 1.0),
        crate::GROUND_HALF_EXTENTS_WIDTH * 1.5,
    );

    let sdl2_window_resource = Sdl2WindowResource::new(sdl2_window);

    let drawable = sdl2_window_resource.drawable_size();
    let viewport_size = ViewportSize::new(drawable.width, drawable.height);
    resources.insert(ViewportResource::new(
        viewport_size,
        camera_resource.position,
        camera_resource.x_half_extents,
    ));
    resources.insert(DebugDrawResource::new());
    resources.insert(EditorDrawResource::new());
    resources.insert(EditorSelectionResource::new(
        registration::create_editor_selection_registry(),
    ));
    resources.insert(EditorInspectRegistryResource::new(
        registration::create_editor_inspector_registry(),
    ));
    resources.insert(ComponentRegistryResource::new(
        registration::create_component_registry(),
    ));
    resources.insert(FpsTextResource::new());
    resources.insert(asset_resource);
    resources.insert(physics_resource);
    resources.insert(camera_resource);
    resources.insert(Sdl2ImguiManagerResource::new(sdl2_imgui.clone()));
    resources.insert(ImguiResource::new(sdl2_imgui.imgui_manager()));
    resources.insert(AppControlResource::new());
    resources.insert(TimeResource::new());
    resources.insert(InputResource::new());
    resources.insert(CanvasDrawResource::default());
    resources.insert(UniverseResource::new(universe));
    resources.insert(Sdl2WindowResource::new(&sdl2_window));
    resources.insert(EditorStateResource::new());

    use minimum_sdl2::input::Sdl2KeyboardKey;
    use sdl2::keyboard::Keycode;
    let keybinds = minimum::resources::editor::Keybinds {
        selection_add: Sdl2KeyboardKey::new(Keycode::LShift).into(),
        selection_subtract: Sdl2KeyboardKey::new(Keycode::LAlt).into(),
        selection_toggle: Sdl2KeyboardKey::new(Keycode::LCtrl).into(),
        tool_translate: Sdl2KeyboardKey::new(Keycode::Num1).into(),
        tool_scale: Sdl2KeyboardKey::new(Keycode::Num2).into(),
        tool_rotate: Sdl2KeyboardKey::new(Keycode::Num3).into(),
        action_quit: Sdl2KeyboardKey::new(Keycode::Escape).into(),
        action_toggle_editor_pause: Sdl2KeyboardKey::new(Keycode::Space).into(),
    };

    resources.insert(minimum::resources::editor::EditorSettingsResource::new(
        keybinds,
    ));
    resources
}
*/

fn add_light_debug_draw(
    resources: &Resources,
    world: &World,
) {
    let mut debug_draw = resources.get_mut::<DebugDraw3DResource>().unwrap();

    let query = <Read<DirectionalLightComponent>>::query();
    for light in query.iter(world) {
        let light_from = glam::Vec3::new(0.0, 0.0, 0.0);
        let light_to = light.direction;

        debug_draw.add_line(light_from, light_to, light.color);
    }

    let query = <(Read<PositionComponent>, Read<PointLightComponent>)>::query();
    for (position, light) in query.iter(world) {
        debug_draw.add_sphere(position.position, 0.25, light.color, 12);
    }

    let query = <(Read<PositionComponent>, Read<SpotLightComponent>)>::query();
    for (position, light) in query.iter(world) {
        let light_from = position.position;
        let light_to = position.position + light.direction;
        let light_direction = (light_to - light_from).normalize();

        debug_draw.add_cone(
            light_from,
            light_from + (light.range * light_direction),
            light.range * light.spotlight_half_angle.tan(),
            light.color,
            8,
        );
    }
}

fn process_input(
    resources: &Resources,
    event_pump: &mut sdl2::EventPump,
) -> bool {
    let imgui_manager = resources.get::<Sdl2ImguiManager>().unwrap();
    for event in event_pump.poll_iter() {
        imgui_manager.handle_event(&event);
        if !imgui_manager.ignore_event(&event) {
            //log::trace!("{:?}", event);
            match event {
                //
                // Halt if the user requests to close the window
                //
                Event::Quit { .. } => return false,

                //
                // Close if the escape key is hit
                //
                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod: _modifiers,
                    ..
                } => {
                    //log::trace!("Key Down {:?} {:?}", keycode, modifiers);
                    if keycode == Keycode::Escape {
                        return false;
                    }

                    if keycode == Keycode::D {
                        let stats = resources
                            .get::<VkDeviceContext>()
                            .unwrap()
                            .allocator()
                            .calculate_stats()
                            .unwrap();
                        println!("{:#?}", stats);
                    }

                    if keycode == Keycode::M {
                        let metrics = resources.get::<ResourceManager>().unwrap().metrics();
                        println!("{:#?}", metrics);
                    }
                }

                _ => {}
            }
        }
    }

    true
}
