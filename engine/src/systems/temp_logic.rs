
// pub fn temp_logic() {
//     {
//         let imgui_resource = resources.get_mut::<ImguiResource>().unwrap();
//         let input_resource = resources.get::<InputResource>().unwrap();
//
//         imgui_resource.with_ui(|ui| {
//             let mouse_position = input_resource.mouse_position();
//
//             imgui::Window::new(imgui::im_str!("Coordinates"))
//                 .position([500.0, 300.0], imgui::Condition::Once)
//                 .size([200.0, 200.0], imgui::Condition::Once)
//                 .build(ui, || {
//                     ui.text(imgui::im_str!("hi"));
//                 }
//                 )
//         });
//     }
// }


use legion::SystemBuilder;
use legion::Schedule;
use minimum::resources::{ImguiResource, InputResource, ViewportResource, DebugDraw3DDepthBehavior};
use crate::features::debug3d::DebugDraw3DResource;

pub fn imgui_draw_mouse_coordinates(schedule: &mut legion::systems::Builder) {
    schedule.add_system(SystemBuilder::new("imgui_draw_mouse_coordinates")
        .write_resource::<ImguiResource>()
        .read_resource::<InputResource>()
        .read_resource::<ViewportResource>()
        .write_resource::<DebugDraw3DResource>()
        .build(
            |_,
            world,
            (imgui_resource, input_resource, viewport_resource, debug_draw_resource),
            _| {
                imgui_resource.with_ui(|ui| {
                    let vp : &ViewportResource = viewport_resource;
                    let mouse_position = input_resource.mouse_position();
                    let world_space00 = vp.viewport_space_to_world_space(mouse_position, 0.0);
                    let world_space099 = vp.viewport_space_to_world_space(mouse_position, 0.99);
                    let world_space10 = vp.viewport_space_to_world_space(mouse_position, 1.0);
                    let view_space = vp.viewport_space_to_view_space(mouse_position);
                    let screen_space = vp.viewport_space_to_screen_space(mouse_position);
                    let norm_space = vp.viewport_space_to_normalized_space(mouse_position);
                    let world_space_ray = vp.viewport_space_to_ray(mouse_position);

                    //debug_draw_resource.add_sphere(world_space00, 0.25, glam::Vec4::new(1.0, 1.0, 0.0, 1.0), DebugDraw3DDepthBehavior::NoDepthTest, 12);
                    //debug_draw_resource.add_sphere(world_space099, 0.25, glam::Vec4::new(0.0, 1.0, 1.0, 1.0), DebugDraw3DDepthBehavior::NoDepthTest, 12);
                    debug_draw_resource.add_sphere(world_space10, 0.25, glam::Vec4::new(0.0, 0.0, 1.0, 1.0), DebugDraw3DDepthBehavior::NoDepthTest, 12);

                    imgui::Window::new(imgui::im_str!("Coordinates"))
                        .position([300.0, 300.0], imgui::Condition::Once)
                        .size([700.0, 300.0], imgui::Condition::Once)
                        .build(ui, || {
                            //ui.text(imgui::im_str!("camera: {:?}", vp.world_space_eye_position()));
                            ui.text(imgui::im_str!("mouse: {:?}", mouse_position));
                            ui.text(imgui::im_str!("world00: {:?}", world_space00));
                            ui.text(imgui::im_str!("world099: {:?}", world_space099));
                            ui.text(imgui::im_str!("world10: {:?}", world_space10));
                            ui.text(imgui::im_str!("world_ray: {:?}", world_space_ray));
                            ui.text(imgui::im_str!("view: {:?}", view_space));
                            ui.text(imgui::im_str!("screen: {:?}", screen_space));
                            ui.text(imgui::im_str!("norm: {:?}", norm_space));
                        }
                    )
                });
            },
        ));
}
