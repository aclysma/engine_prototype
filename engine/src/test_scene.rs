use legion::{Resources, World};
use minimum::resources::AssetResource;
use glam::f32::Vec3;
use crate::features::sprite::{SpriteRenderNodeSet, SpriteRenderNode};
use renderer::visibility::{DynamicVisibilityNodeSet, DynamicAabbVisibilityNode};
use crate::components::{
    SpriteComponent, PointLightComponent, SpotLightComponent, DirectionalLightComponent,
};
use renderer::assets::ImageAsset;
use crate::features::mesh::{MeshRenderNodeSet, MeshRenderNode};
use atelier_assets::core::asset_uuid;
use atelier_assets::core as atelier_core;
use atelier_assets::core::AssetUuid;
use crate::components::MeshComponent;
use crate::assets::gltf::MeshAsset;
use minimum::components::{TransformComponentDef, TransformComponent};

fn begin_load_asset<T>(
    asset_uuid: AssetUuid,
    asset_resource: &AssetResource,
) -> atelier_assets::loader::handle::Handle<T> {
    use atelier_assets::loader::Loader;
    let load_handle = asset_resource.loader().add_ref(asset_uuid);
    atelier_assets::loader::handle::Handle::<T>::new(asset_resource.tx().clone(), load_handle)
}

pub fn populate_test_sprite_entities(
    resources: &mut Resources,
    world: &mut World,
) {
    let sprite_image = {
        let asset_resource = resources.get::<AssetResource>().unwrap();
        begin_load_asset::<ImageAsset>(
            asset_uuid!("7c42f3bc-e96b-49f6-961b-5bfc799dee50"),
            &asset_resource,
        )
    };

    let sprites = ["sprite1", "sprite2", "sprite3"];
    for i in 0..1 {
        let position = Vec3::new(((i / 10) * 25) as f32, ((i % 10) * 25) as f32, 0.0);
        //let alpha = if i % 7 == 0 { 0.50 } else { 1.0 };
        let alpha = 1.0;
        let _sprite = sprites[i % sprites.len()];

        let mut sprite_render_nodes = resources.get_mut::<SpriteRenderNodeSet>().unwrap();
        let mut dynamic_visibility_node_set =
            resources.get_mut::<DynamicVisibilityNodeSet>().unwrap();

        let render_node = sprite_render_nodes.register_sprite(SpriteRenderNode {
            position,
            alpha,
            image: sprite_image.clone(),
        });

        let aabb_info = DynamicAabbVisibilityNode {
            handle: render_node.as_raw_generic_handle(),
            // aabb bounds
        };

        let visibility_node = dynamic_visibility_node_set.register_dynamic_aabb(aabb_info);

        let mut transform_component = TransformComponentDef::default();
        *transform_component.position_mut() = position;
        let sprite_component = SpriteComponent {
            render_node,
            visibility_node,
            alpha,
            image: sprite_image.clone(),
        };

        world.extend((0..1).map(|_| (transform_component.clone(), sprite_component.clone())));
    }
}

pub fn populate_test_mesh_entities(
    resources: &mut Resources,
    world: &mut World,
) {
    let mesh = {
        let asset_resource = resources.get::<AssetResource>().unwrap();
        begin_load_asset::<MeshAsset>(
            asset_uuid!("ffc9b240-0a17-4ff4-bb7d-72d13cc6e261"),
            &asset_resource,
        )
    };

    for i in 0..1 {
        let position = Vec3::new(((i / 10) * 3) as f32, ((i % 10) * 3) as f32, 0.0);

        let mut mesh_render_nodes = resources.get_mut::<MeshRenderNodeSet>().unwrap();
        let mut dynamic_visibility_node_set =
            resources.get_mut::<DynamicVisibilityNodeSet>().unwrap();

        let transform_component = TransformComponent::from_position(position);

        let render_node = mesh_render_nodes.register_mesh(MeshRenderNode {
            transform: transform_component.transform(),
            mesh: Some(mesh.clone()),
        });

        let aabb_info = DynamicAabbVisibilityNode {
            handle: render_node.as_raw_generic_handle(),
            // aabb bounds
        };

        let visibility_node = dynamic_visibility_node_set.register_dynamic_aabb(aabb_info);
        let mesh_component = MeshComponent {
            render_node,
            visibility_node,
            mesh: Some(mesh.clone()),
        };

        world.extend(vec![(transform_component, mesh_component)]);
    }
}

pub fn populate_test_lights(
    resources: &mut Resources,
    world: &mut World,
) {
    add_point_light(
        resources,
        world,
        glam::Vec3::new(-3.0, -3.0, 3.0),
        PointLightComponent {
            color: [1.0, 1.0, 1.0].into(),
            intensity: 130.0,
            range: 25.0,
        },
    );

    add_point_light(
        resources,
        world,
        glam::Vec3::new(-3.0, 3.0, 3.0),
        PointLightComponent {
            color: [1.0, 1.0, 1.0].into(),
            intensity: 130.0,
            range: 25.0,
        },
    );

    let light_from = glam::Vec3::new(-3.0, -3.0, 0.0);
    let light_to = glam::Vec3::new(0.0, 0.0, 0.0);
    let light_direction = (light_to - light_from).normalize();
    add_spot_light(
        resources,
        world,
        light_from,
        SpotLightComponent {
            direction: light_direction.into(),
            spotlight_half_angle: 10.0 * (std::f32::consts::PI / 180.0),
            range: 8.0,
            color: [1.0, 1.0, 1.0].into(),
            intensity: 1000.0,
        },
    );

    let light_from = glam::Vec3::new(5.0, 5.0, 5.0);
    let light_to = glam::Vec3::new(0.0, 0.0, 0.0);
    let light_direction = (light_to - light_from).normalize();
    add_directional_light(
        resources,
        world,
        DirectionalLightComponent {
            direction: light_direction.into(),
            intensity: 5.0,
            color: [1.0, 1.0, 1.0].into(),
        },
    );
}

fn add_directional_light(
    _resources: &mut Resources,
    world: &mut World,
    light_component: DirectionalLightComponent,
) {
    world.extend(vec![(light_component,)]);
}

fn add_spot_light(
    _resources: &mut Resources,
    world: &mut World,
    position: glam::Vec3,
    light_component: SpotLightComponent,
) {
    let transform_component = TransformComponent::from_position(position);

    world.extend(vec![(transform_component, light_component)]);
}

fn add_point_light(
    _resources: &mut Resources,
    world: &mut World,
    position: glam::Vec3,
    light_component: PointLightComponent,
) {
    let transform_component = TransformComponent::from_position(position);

    world.extend(vec![(transform_component, light_component)]);
}
