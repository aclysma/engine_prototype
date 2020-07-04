use legion::prelude::*;
use renderer::assets::ResourceManager;
use crate::game_resource_manager::GameResourceManager;
use crate::components::{
    DirectionalLightComponent, PointLightComponent, SpotLightComponent,
};
use minimum::resources::{DebugDraw3DResource, DebugDraw3DDepthBehavior};
use minimum::components::PositionComponent;

pub fn add_light_debug_draw() -> Box<dyn Schedulable> {
    SystemBuilder::new("quit_if_escape_pressed")
        .write_resource::<DebugDraw3DResource>()
        .with_query(<Read<DirectionalLightComponent>>::query())
        .with_query(<(Read<PositionComponent>, Read<PointLightComponent>)>::query())
        .with_query(<(Read<PositionComponent>, Read<SpotLightComponent>)>::query())
        .build(
            |_,
             world,
             debug_draw,
             (directional_light_query, point_light_query, spot_light_query)| {
                for light in directional_light_query.iter(world) {
                    let light_from = glam::Vec3::new(0.0, 0.0, 0.0);
                    let light_to = light.direction;

                    debug_draw.add_line(light_from, light_to, light.color, DebugDraw3DDepthBehavior::Normal);
                }

                for (position, light) in point_light_query.iter(world) {
                    debug_draw.add_sphere(*position.position, 0.25, light.color, DebugDraw3DDepthBehavior::Normal, 12);
                }

                for (position, light) in spot_light_query.iter(world) {
                    let light_from = *position.position;
                    let light_to = *position.position + light.direction;
                    let light_direction = (light_to - light_from).normalize();

                    debug_draw.add_cone(
                        light_from,
                        light_from + (light.range * light_direction),
                        light.range * light.spotlight_half_angle.tan(),
                        light.color,
                        DebugDraw3DDepthBehavior::Normal,
                        8,
                    );
                }
            },
        )
}
