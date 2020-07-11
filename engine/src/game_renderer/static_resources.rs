use minimum::resources::AssetResource;
use renderer::assets::resources::ResourceManager;
use atelier_assets::loader::handle::Handle;
use atelier_assets::core::asset_uuid;
use atelier_assets::core::AssetUuid;
use atelier_assets::loader::LoadStatus;
use atelier_assets::core as atelier_core;
use ash::prelude::VkResult;
use atelier_assets::loader::handle::AssetHandle;
use renderer::assets::MaterialAsset;
use legion::prelude::Resources;

fn begin_load_asset<T>(
    asset_uuid: AssetUuid,
    resources: &Resources,
) -> atelier_assets::loader::handle::Handle<T> {
    use atelier_assets::loader::Loader;
    let asset_resource = resources.get::<AssetResource>().unwrap();
    let load_handle = asset_resource.loader().add_ref(asset_uuid);
    atelier_assets::loader::handle::Handle::<T>::new(asset_resource.tx().clone(), load_handle)
}

fn wait_for_asset_to_load<T>(
    asset_handle: &atelier_assets::loader::handle::Handle<T>,
    resources: &Resources,
    asset_name: &str,
) -> VkResult<()> {
    let mut asset_resource = resources.get_mut::<AssetResource>().unwrap();
    loop {
        asset_resource.update(resources);
        //resource_manager.update_resources()?;
        match asset_handle.load_status(asset_resource.loader()) {
            LoadStatus::NotRequested => {
                unreachable!();
            }
            LoadStatus::Loading => {
                log::info!(
                    "blocked waiting for asset to load {} {:?}",
                    asset_name,
                    asset_handle
                );
                std::thread::sleep(std::time::Duration::from_millis(10));
                // keep waiting
            }
            LoadStatus::Loaded => {
                break Ok(());
            }
            LoadStatus::Unloading => unreachable!(),
            LoadStatus::DoesNotExist => {
                println!("Essential asset not found");
            }
            LoadStatus::Error(err) => {
                println!("Error loading essential asset {:?}", err);
            }
        }
    }
}

pub struct GameRendererStaticResources {
    pub sprite_material: Handle<MaterialAsset>,
    pub debug_material_3d: Handle<MaterialAsset>,
    pub debug_material_3d_no_depth: Handle<MaterialAsset>,
    pub debug_material_2d: Handle<MaterialAsset>,
    pub mesh_material: Handle<MaterialAsset>,
    pub bloom_extract_material: Handle<MaterialAsset>,
    pub bloom_blur_material: Handle<MaterialAsset>,
    pub bloom_combine_material: Handle<MaterialAsset>,
    pub imgui_material: Handle<MaterialAsset>,
}

impl GameRendererStaticResources {
    pub fn new(
        resources: &Resources,
    ) -> VkResult<Self> {
        //
        // Sprite resources
        //
        let sprite_material = begin_load_asset::<MaterialAsset>(
            asset_uuid!("f8c4897e-7c1d-4736-93b7-f2deda158ec7"),
            resources,
        );

        //
        // Debug resources
        //
        let debug_material_3d = begin_load_asset::<MaterialAsset>(
            asset_uuid!("11d3b144-f564-42c9-b31f-82c8a938bf85"),
            resources,
        );

        //
        // Debug resources
        //
        let debug_material_3d_no_depth = begin_load_asset::<MaterialAsset>(
            asset_uuid!("31c5d4f7-e330-4a02-9544-7cc6db2c4fb5"),
            resources,
        );

        //
        // Bloom extract resources
        //
        let bloom_extract_material = begin_load_asset::<MaterialAsset>(
            asset_uuid!("822c8e08-2720-4002-81da-fd9c4d61abdd"),
            resources,
        );

        //
        // Bloom blur resources
        //
        let bloom_blur_material = begin_load_asset::<MaterialAsset>(
            asset_uuid!("22aae4c1-fd0f-414a-9de1-7f68bdf1bfb1"),
            resources,
        );

        //
        // Bloom combine resources
        //
        let bloom_combine_material = begin_load_asset::<MaterialAsset>(
            asset_uuid!("256e6a2d-669b-426b-900d-3bcc4249a063"),
            resources,
        );

        //
        // Mesh resources
        //
        let mesh_material = begin_load_asset::<MaterialAsset>(
            asset_uuid!("267e0388-2611-441c-9c78-2d39d1bd3cf1"),
            resources,
        );

        //
        // ImGui resources
        //
        let imgui_material = begin_load_asset::<MaterialAsset>(
            asset_uuid!("b1cd2431-5cf8-4e9c-b7f0-569ba74e0981"),
            resources,
        );

        wait_for_asset_to_load(
            &sprite_material,
            resources,
            "sprite_material",
        )?;

        wait_for_asset_to_load(
            &debug_material_3d,
            resources,
            "debug material",
        )?;

        wait_for_asset_to_load(
            &debug_material_3d_no_depth,
            resources,
            "debug material no depth",
        )?;

        wait_for_asset_to_load(
            &bloom_extract_material,
            resources,
            "bloom extract material",
        )?;

        wait_for_asset_to_load(
            &bloom_blur_material,
            resources,
            "bloom blur material",
        )?;

        wait_for_asset_to_load(
            &bloom_combine_material,
            resources,
            "bloom combine material",
        )?;

        wait_for_asset_to_load(
            &mesh_material,
            resources,
            "mesh material",
        )?;

        wait_for_asset_to_load(
            &imgui_material,
            resources,
            "imgui material",
        )?;

        let debug_material_2d = debug_material_3d_no_depth.clone();

        Ok(GameRendererStaticResources {
            sprite_material,
            debug_material_3d,
            debug_material_3d_no_depth,
            debug_material_2d,
            mesh_material,
            bloom_extract_material,
            bloom_blur_material,
            bloom_combine_material,
            imgui_material,
        })
    }
}
