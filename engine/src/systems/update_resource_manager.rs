use legion::prelude::*;
use renderer::assets::ResourceManager;
use crate::game_resource_manager::GameResourceManager;

// pub fn update_resource_manager() -> Box<dyn Schedulable> {
//     SystemBuilder::new("quit_if_escape_pressed")
//         .write_resource::<ResourceManager>()
//         .write_resource::<GameResourceManager>()
//         .build(|_, _, (resource_manager, game_resource_manager), _| {
//             resource_manager.update_resources().unwrap();
//             game_resource_manager
//                 .update_resources(&*resource_manager)
//                 .unwrap();
//         })
// }
