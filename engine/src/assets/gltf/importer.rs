use atelier_assets::core::{AssetUuid, AssetRef};
use atelier_assets::importer::{Error, ImportedAsset, Importer, ImporterValue};
use serde::{Deserialize, Serialize};
use type_uuid::*;
use std::io::Read;
use std::convert::TryInto;
use gltf::image::Data as GltfImageData;
use gltf::buffer::Data as GltfBufferData;
use fnv::FnvHashMap;
use atelier_assets::loader::handle::Handle;
use crate::assets::gltf::{
    GltfMaterialAsset, MeshAssetData, MeshPartData, MeshVertex, GltfMaterialDataShaderParam,
    MeshAsset,
};
use renderer::assets::assets::{ImageAssetData, ColorSpace};
use renderer::assets::assets::BufferAssetData;
use renderer::assets::push_buffer::PushBuffer;
use atelier_assets::loader::handle::SerdeContext;
use renderer::assets::assets::{MaterialInstanceAssetData, MaterialInstanceSlotAssignment};
use std::str::FromStr;
use serde::export::Formatter;
use renderer::assets::ImageAsset;
use renderer::assets::MaterialInstanceAsset;
use renderer::assets::BufferAsset;
use renderer::assets::MaterialAsset;
use minimum::math::BoundingAabb;
use itertools::Itertools;
use legion::*;
use minimum::pipeline::PrefabAsset;
use minimum::components::{TransformComponentDef, EditorMetadataComponent, TransformComponent};
use crate::components::{
    MeshComponentDef, DirectionalLightComponent, PointLightComponent, SpotLightComponent,
};
use legion_prefab::{Prefab};
use gltf::camera::Projection;

#[derive(Debug)]
struct GltfImportError {
    error_message: String,
}

impl GltfImportError {
    pub fn new(error_message: &str) -> Self {
        GltfImportError {
            error_message: error_message.to_string(),
        }
    }
}

impl std::error::Error for GltfImportError {}

impl std::fmt::Display for GltfImportError {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum GltfObjectId {
    Name(String),
    Index(usize),
}

struct ImageToImport {
    id: GltfObjectId,
    asset: ImageAssetData,
}

struct MaterialToImport {
    id: GltfObjectId,
    asset: GltfMaterialAsset,
}

struct MeshToImport {
    id: GltfObjectId,
    asset: MeshAssetData,
}

struct BufferToImport {
    id: GltfObjectId,
    asset: BufferAssetData,
}

struct PrefabToImport {
    id: GltfObjectId,
    asset: PrefabAsset,
}

// The asset state is stored in this format using Vecs
#[derive(TypeUuid, Serialize, Deserialize, Default, Clone)]
#[uuid = "807c83b3-c24c-4123-9580-5f9c426260b4"]
pub struct GltfImporterStateStable {
    // Asset UUIDs for imported image by name. We use vecs here so we can sort by UUID for
    // deterministic output
    buffer_asset_uuids: Vec<(GltfObjectId, AssetUuid)>,
    image_asset_uuids: Vec<(GltfObjectId, AssetUuid)>,
    material_asset_uuids: Vec<(GltfObjectId, AssetUuid)>,
    material_instance_asset_uuids: Vec<(GltfObjectId, AssetUuid)>,
    mesh_asset_uuids: Vec<(GltfObjectId, AssetUuid)>,
    prefab_asset_uuids: Vec<(GltfObjectId, AssetUuid)>,
}

impl From<GltfImporterStateUnstable> for GltfImporterStateStable {
    fn from(other: GltfImporterStateUnstable) -> Self {
        let mut stable = GltfImporterStateStable::default();
        stable.buffer_asset_uuids = other
            .buffer_asset_uuids
            .into_iter()
            .sorted_by_key(|(id, _uuid)| id.clone())
            .collect();
        stable.image_asset_uuids = other
            .image_asset_uuids
            .into_iter()
            .sorted_by_key(|(id, _uuid)| id.clone())
            .collect();
        stable.material_asset_uuids = other
            .material_asset_uuids
            .into_iter()
            .sorted_by_key(|(id, _uuid)| id.clone())
            .collect();
        stable.material_instance_asset_uuids = other
            .material_instance_asset_uuids
            .into_iter()
            .sorted_by_key(|(id, _uuid)| id.clone())
            .collect();
        stable.mesh_asset_uuids = other
            .mesh_asset_uuids
            .into_iter()
            .sorted_by_key(|(id, _uuid)| id.clone())
            .collect();
        stable.prefab_asset_uuids = other
            .prefab_asset_uuids
            .into_iter()
            .sorted_by_key(|(id, _uuid)| id.clone())
            .collect();
        stable
    }
}

// When processing the asset, we use hashmaps for faster lookups
#[derive(Default)]
pub struct GltfImporterStateUnstable {
    buffer_asset_uuids: FnvHashMap<GltfObjectId, AssetUuid>,
    image_asset_uuids: FnvHashMap<GltfObjectId, AssetUuid>,
    material_asset_uuids: FnvHashMap<GltfObjectId, AssetUuid>,
    material_instance_asset_uuids: FnvHashMap<GltfObjectId, AssetUuid>,
    mesh_asset_uuids: FnvHashMap<GltfObjectId, AssetUuid>,
    prefab_asset_uuids: FnvHashMap<GltfObjectId, AssetUuid>,
}

impl From<GltfImporterStateStable> for GltfImporterStateUnstable {
    fn from(other: GltfImporterStateStable) -> Self {
        let mut unstable = GltfImporterStateUnstable::default();
        unstable.buffer_asset_uuids = other.buffer_asset_uuids.into_iter().collect();
        unstable.image_asset_uuids = other.image_asset_uuids.into_iter().collect();
        unstable.material_asset_uuids = other.material_asset_uuids.into_iter().collect();
        unstable.material_instance_asset_uuids =
            other.material_instance_asset_uuids.into_iter().collect();
        unstable.mesh_asset_uuids = other.mesh_asset_uuids.into_iter().collect();
        unstable.prefab_asset_uuids = other.prefab_asset_uuids.into_iter().collect();
        unstable
    }
}

#[derive(TypeUuid)]
#[uuid = "fc9ae812-110d-4daf-9223-e87b40966c6b"]
pub struct GltfImporter;
impl Importer for GltfImporter {
    fn version_static() -> u32
    where
        Self: Sized,
    {
        27
    }

    fn version(&self) -> u32 {
        Self::version_static()
    }

    type Options = ();

    type State = GltfImporterStateStable;

    /// Reads the given bytes and produces assets.
    fn import(
        &self,
        source: &mut dyn Read,
        _options: Self::Options,
        stable_state: &mut Self::State,
    ) -> atelier_assets::importer::Result<ImporterValue> {
        let mut unstable_state: GltfImporterStateUnstable = stable_state.clone().into();

        //
        // Load the GLTF file
        //
        let mut bytes = Vec::new();
        source.read_to_end(&mut bytes)?;
        let result = gltf::import_slice(&bytes);
        if let Err(err) = result {
            log::error!("GLTF Import error: {:?}", err);
            return Err(Error::Boxed(Box::new(err)));
        }

        let (doc, buffers, images) = result.unwrap();

        // Accumulate everything we will import in this list
        let mut imported_assets = Vec::new();

        // Iterate all materials and determine whether the texture should be treated as Srgb or linear
        let image_color_space_assignments =
            build_image_color_space_assignments_from_materials(&doc);

        //
        // Images
        //
        let images_to_import =
            extract_images_to_import(&doc, &buffers, &images, &image_color_space_assignments);
        let mut image_index_to_handle = vec![];
        for image_to_import in images_to_import {
            // Find the UUID associated with this image or create a new one
            let image_uuid = *unstable_state
                .image_asset_uuids
                .entry(image_to_import.id.clone())
                .or_insert_with(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));

            let image_handle = SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
                let load_handle = loader_info_provider
                    .get_load_handle(&AssetRef::Uuid(image_uuid))
                    .unwrap();
                Handle::<ImageAsset>::new(ref_op_sender.clone(), load_handle)
            });

            // Push the UUID into the list so that we have an O(1) lookup for image index to UUID
            image_index_to_handle.push(image_handle);

            let mut search_tags: Vec<(String, Option<String>)> = vec![];
            if let GltfObjectId::Name(name) = &image_to_import.id {
                search_tags.push(("image_name".to_string(), Some(name.clone())));
            }

            log::debug!("Importing image uuid {:?}", image_uuid);

            // Create the asset
            imported_assets.push(ImportedAsset {
                id: image_uuid,
                search_tags,
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(image_to_import.asset),
            });
        }

        //
        // GLTF Material (which we may not end up needing)
        //
        let materials_to_import =
            extract_materials_to_import(&doc, &buffers, &images, &image_index_to_handle);
        let mut material_index_to_handle = vec![];
        for material_to_import in &materials_to_import {
            // Find the UUID associated with this image or create a new one
            let material_uuid = *unstable_state
                .material_asset_uuids
                .entry(material_to_import.id.clone())
                .or_insert_with(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));

            let material_handle =
                SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
                    let load_handle = loader_info_provider
                        .get_load_handle(&AssetRef::Uuid(material_uuid))
                        .unwrap();
                    Handle::<GltfMaterialAsset>::new(ref_op_sender.clone(), load_handle)
                });

            // Push the UUID into the list so that we have an O(1) lookup for image index to UUID
            material_index_to_handle.push(material_handle);

            let mut search_tags: Vec<(String, Option<String>)> = vec![];
            if let GltfObjectId::Name(name) = &material_to_import.id {
                search_tags.push(("material_name".to_string(), Some(name.clone())));
            }
            log::debug!("Importing material uuid {:?}", material_uuid);

            // Create the asset
            imported_assets.push(ImportedAsset {
                id: material_uuid,
                search_tags,
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(material_to_import.asset.clone()),
            });
        }

        let material_handle = SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
            let material_uuid_str = "267e0388-2611-441c-9c78-2d39d1bd3cf1";
            let material_uuid =
                AssetUuid(*uuid::Uuid::from_str(material_uuid_str).unwrap().as_bytes());

            let material_load_handle = loader_info_provider
                .get_load_handle(&AssetRef::Uuid(material_uuid))
                .unwrap();
            Handle::<MaterialAsset>::new(ref_op_sender.clone(), material_load_handle)
        });

        let null_image_handle = SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
            let material_uuid_str = "be831a21-f4f6-45d4-b9eb-e1bb6fc19d22";
            let material_uuid =
                AssetUuid(*uuid::Uuid::from_str(material_uuid_str).unwrap().as_bytes());

            let material_load_handle = loader_info_provider
                .get_load_handle(&AssetRef::Uuid(material_uuid))
                .unwrap();
            Handle::<ImageAsset>::new(ref_op_sender.clone(), material_load_handle)
        });

        //
        // Material instance
        //
        let mut material_instance_index_to_handle = vec![];
        for material_to_import in &materials_to_import {
            let material_instance_uuid = *unstable_state
                .material_instance_asset_uuids
                .entry(material_to_import.id.clone())
                .or_insert_with(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));

            let material_instance_handle =
                SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
                    let load_handle = loader_info_provider
                        .get_load_handle(&AssetRef::Uuid(material_instance_uuid))
                        .unwrap();
                    Handle::<MaterialInstanceAsset>::new(ref_op_sender.clone(), load_handle)
                });

            // Push the UUID into the list so that we have an O(1) lookup for image index to UUID
            material_instance_index_to_handle.push(material_instance_handle);

            let mut search_tags: Vec<(String, Option<String>)> = vec![];
            if let GltfObjectId::Name(name) = &material_to_import.id {
                search_tags.push(("material_name".to_string(), Some(name.clone())));
            }

            let mut slot_assignments = vec![];

            let material_data_shader_param: GltfMaterialDataShaderParam =
                material_to_import.asset.material_data.clone().into();
            slot_assignments.push(MaterialInstanceSlotAssignment {
                slot_name: "per_material_data".to_string(),
                image: None,
                sampler: None,
                buffer_data: Some(
                    renderer::vulkan::util::any_as_bytes(&material_data_shader_param).into(),
                ),
            });

            fn push_image_slot_assignment(
                slot_name: &str,
                slot_assignments: &mut Vec<MaterialInstanceSlotAssignment>,
                image: &Option<Handle<ImageAsset>>,
                default_image: &Handle<ImageAsset>,
            ) {
                slot_assignments.push(MaterialInstanceSlotAssignment {
                    slot_name: slot_name.to_string(),
                    image: Some(image.as_ref().map_or(default_image, |x| x).clone()),
                    sampler: None,
                    buffer_data: None,
                });
            }

            push_image_slot_assignment(
                "base_color_texture",
                &mut slot_assignments,
                &material_to_import.asset.base_color_texture,
                &null_image_handle,
            );
            push_image_slot_assignment(
                "metallic_roughness_texture",
                &mut slot_assignments,
                &material_to_import.asset.metallic_roughness_texture,
                &null_image_handle,
            );
            push_image_slot_assignment(
                "normal_texture",
                &mut slot_assignments,
                &material_to_import.asset.normal_texture,
                &null_image_handle,
            );
            push_image_slot_assignment(
                "occlusion_texture",
                &mut slot_assignments,
                &material_to_import.asset.occlusion_texture,
                &null_image_handle,
            );
            push_image_slot_assignment(
                "emissive_texture",
                &mut slot_assignments,
                &material_to_import.asset.emissive_texture,
                &null_image_handle,
            );

            let material_instance_asset = MaterialInstanceAssetData {
                material: material_handle.clone(),
                slot_assignments,
            };

            log::debug!(
                "Importing material instance uuid {:?}",
                material_instance_uuid
            );

            // Create the asset
            imported_assets.push(ImportedAsset {
                id: material_instance_uuid,
                search_tags,
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(material_instance_asset),
            });
        }

        //
        // Meshes
        //
        let (meshes_to_import, buffers_to_import) = extract_meshes_to_import(
            &mut unstable_state,
            &doc,
            &buffers,
            &material_index_to_handle,
            &material_instance_index_to_handle,
        )?;

        let mut buffer_index_to_handle = vec![];
        for buffer_to_import in buffers_to_import {
            // Find the UUID associated with this image or create a new one
            let buffer_uuid = *unstable_state
                .buffer_asset_uuids
                .entry(buffer_to_import.id.clone())
                .or_insert_with(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));

            let buffer_handle = SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
                let load_handle = loader_info_provider
                    .get_load_handle(&AssetRef::Uuid(buffer_uuid))
                    .unwrap();
                Handle::<GltfMaterialAsset>::new(ref_op_sender.clone(), load_handle)
            });

            // Push the UUID into the list so that we have an O(1) lookup for image index to UUID
            buffer_index_to_handle.push(buffer_handle);

            log::debug!("Importing buffer uuid {:?}", buffer_uuid);

            // Create the asset
            imported_assets.push(ImportedAsset {
                id: buffer_uuid,
                search_tags: vec![],
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(buffer_to_import.asset),
            });
        }

        //let mut mesh_index_to_uuid_lookup = vec![];
        let mut mesh_index_to_handle = vec![];
        for mesh_to_import in meshes_to_import {
            // Find the UUID associated with this image or create a new one
            let mesh_uuid = *unstable_state
                .mesh_asset_uuids
                .entry(mesh_to_import.id.clone())
                .or_insert_with(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));

            let mesh_handle = SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
                let load_handle = loader_info_provider
                    .get_load_handle(&AssetRef::Uuid(mesh_uuid))
                    .unwrap();
                Handle::<MeshAsset>::new(ref_op_sender.clone(), load_handle)
            });

            let mut search_tags: Vec<(String, Option<String>)> = vec![];
            if let GltfObjectId::Name(name) = &mesh_to_import.id {
                search_tags.push(("mesh_name".to_string(), Some(name.clone())));
            }

            // Push the UUID into the list so that we have an O(1) lookup for image index to UUID
            mesh_index_to_handle.push(mesh_handle);

            log::debug!("Importing mesh uuid {:?}", mesh_uuid);

            // Create the asset
            imported_assets.push(ImportedAsset {
                id: mesh_uuid,
                search_tags,
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(mesh_to_import.asset),
            });
        }

        //
        // Scenes
        //
        let prefabs_to_import = extract_prefabs_to_import(
            &doc,
            mesh_index_to_handle.as_slice(),
            &mut unstable_state.prefab_asset_uuids,
        );
        for prefab_to_import in prefabs_to_import {
            // Find the UUID associated with this image or create a new one
            let prefab_uuid = AssetUuid(prefab_to_import.asset.prefab.prefab_id());

            SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
                let load_handle = loader_info_provider
                    .get_load_handle(&AssetRef::Uuid(prefab_uuid))
                    .unwrap();
                Handle::<PrefabAsset>::new(ref_op_sender.clone(), load_handle)
            });

            let mut search_tags: Vec<(String, Option<String>)> = vec![];
            if let GltfObjectId::Name(name) = &prefab_to_import.id {
                search_tags.push(("scene_name".to_string(), Some(name.clone())));
            }

            log::debug!("Importing prefab uuid {:?}", prefab_uuid);

            // Create the asset
            imported_assets.push(ImportedAsset {
                id: prefab_uuid,
                search_tags,
                build_deps: vec![],
                load_deps: vec![],
                build_pipeline: None,
                asset_data: Box::new(prefab_to_import.asset),
            });
        }

        *stable_state = unstable_state.into();

        Ok(ImporterValue {
            assets: imported_assets,
        })
    }
}

fn extract_images_to_import(
    doc: &gltf::Document,
    _buffers: &Vec<GltfBufferData>,
    images: &Vec<GltfImageData>,
    image_color_space_assignments: &FnvHashMap<usize, ColorSpace>,
) -> Vec<ImageToImport> {
    let mut images_to_import = Vec::with_capacity(images.len());
    for image in doc.images() {
        let image_data = &images[image.index()];

        // Convert it to standard RGBA format
        use gltf::image::Format;
        use image::buffer::ConvertBuffer;
        let converted_image: image::RgbaImage = match image_data.format {
            Format::R8 => image::ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_vec(
                image_data.width,
                image_data.height,
                image_data.pixels.clone(),
            )
            .unwrap()
            .convert(),
            Format::R8G8 => image::ImageBuffer::<image::LumaA<u8>, Vec<u8>>::from_vec(
                image_data.width,
                image_data.height,
                image_data.pixels.clone(),
            )
            .unwrap()
            .convert(),
            Format::R8G8B8 => image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_vec(
                image_data.width,
                image_data.height,
                image_data.pixels.clone(),
            )
            .unwrap()
            .convert(),
            Format::R8G8B8A8 => image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_vec(
                image_data.width,
                image_data.height,
                image_data.pixels.clone(),
            )
            .unwrap()
            .convert(),
            Format::B8G8R8 => image::ImageBuffer::<image::Bgr<u8>, Vec<u8>>::from_vec(
                image_data.width,
                image_data.height,
                image_data.pixels.clone(),
            )
            .unwrap()
            .convert(),
            Format::B8G8R8A8 => image::ImageBuffer::<image::Bgra<u8>, Vec<u8>>::from_vec(
                image_data.width,
                image_data.height,
                image_data.pixels.clone(),
            )
            .unwrap()
            .convert(),
            Format::R16 => {
                unimplemented!();
            }
            Format::R16G16 => {
                unimplemented!();
            }
            Format::R16G16B16 => {
                unimplemented!();
            }
            Format::R16G16B16A16 => {
                unimplemented!();
            }
        };

        let color_space = *image_color_space_assignments
            .get(&image.index())
            .unwrap_or(&ColorSpace::Linear);
        log::info!(
            "Choosing color space {:?} for image index {}",
            color_space,
            image.index()
        );

        let asset = ImageAssetData {
            data: converted_image.to_vec(),
            width: image_data.width,
            height: image_data.height,
            color_space,
        };
        let id = image
            .name()
            .map(|s| GltfObjectId::Name(s.to_string()))
            .unwrap_or(GltfObjectId::Index(image.index()));

        let image_to_import = ImageToImport { id, asset };

        // Verify that we iterate images in order so that our resulting assets are in order
        assert!(image.index() == images_to_import.len());
        log::debug!(
            "Importing Texture name: {:?} index: {} width: {} height: {} bytes: {}",
            image.name(),
            image.index(),
            image_to_import.asset.width,
            image_to_import.asset.height,
            image_to_import.asset.data.len()
        );

        images_to_import.push(image_to_import);
    }

    images_to_import
}

fn build_image_color_space_assignments_from_materials(
    doc: &gltf::Document
) -> FnvHashMap<usize, ColorSpace> {
    let mut image_color_space_assignments = FnvHashMap::default();

    for material in doc.materials() {
        let pbr_metallic_roughness = &material.pbr_metallic_roughness();

        if let Some(texture) = pbr_metallic_roughness.base_color_texture() {
            image_color_space_assignments
                .insert(texture.texture().source().index(), ColorSpace::Srgb);
        }

        if let Some(texture) = pbr_metallic_roughness.metallic_roughness_texture() {
            image_color_space_assignments
                .insert(texture.texture().source().index(), ColorSpace::Linear);
        }

        if let Some(texture) = material.normal_texture() {
            image_color_space_assignments
                .insert(texture.texture().source().index(), ColorSpace::Linear);
        }

        if let Some(texture) = material.occlusion_texture() {
            image_color_space_assignments
                .insert(texture.texture().source().index(), ColorSpace::Srgb);
        }

        if let Some(texture) = material.emissive_texture() {
            image_color_space_assignments
                .insert(texture.texture().source().index(), ColorSpace::Srgb);
        }
    }

    image_color_space_assignments
}

fn extract_materials_to_import(
    doc: &gltf::Document,
    _buffers: &Vec<GltfBufferData>,
    _images: &Vec<GltfImageData>,
    image_index_to_handle: &[Handle<ImageAsset>],
) -> Vec<MaterialToImport> {
    let mut materials_to_import = Vec::with_capacity(doc.materials().len());

    for material in doc.materials() {
        /*
                let mut material_data = GltfMaterialData {
                    base_color_factor: [f32; 4], // default: 1,1,1,1
                    emissive_factor: [f32; 3],
                    metallic_factor: f32, //default: 1,
                    roughness_factor: f32, // default: 1,
                    normal_texture_scale: f32, // default: 1
                    occlusion_texture_strength: f32, // default 1
                    alpha_cutoff: f32, // default 0.5
                }

                let material_asset = GltfMaterialAsset {
                    material_data,
                    base_color_factor: base_color,
                    base_color_texture: base_color_texture.clone(),
                    metallic_roughness_texture: None,
                    normal_texture: None,
                    occlusion_texture: None,
                    emissive_texture: None,
                };
        */
        let mut material_asset = GltfMaterialAsset::default();

        let pbr_metallic_roughness = &material.pbr_metallic_roughness();
        material_asset.material_data.base_color_factor = pbr_metallic_roughness.base_color_factor();
        material_asset.material_data.emissive_factor = material.emissive_factor();
        material_asset.material_data.metallic_factor = pbr_metallic_roughness.metallic_factor();
        material_asset.material_data.roughness_factor = pbr_metallic_roughness.roughness_factor();
        material_asset.material_data.normal_texture_scale =
            material.normal_texture().map_or(1.0, |x| x.scale());
        material_asset.material_data.occlusion_texture_strength =
            material.occlusion_texture().map_or(1.0, |x| x.strength());
        material_asset.material_data.alpha_cutoff = material.alpha_cutoff();

        material_asset.base_color_texture = pbr_metallic_roughness
            .base_color_texture()
            .map(|texture| image_index_to_handle[texture.texture().source().index()].clone());
        material_asset.metallic_roughness_texture = pbr_metallic_roughness
            .metallic_roughness_texture()
            .map(|texture| image_index_to_handle[texture.texture().source().index()].clone());
        material_asset.normal_texture = material
            .normal_texture()
            .map(|texture| image_index_to_handle[texture.texture().source().index()].clone());
        material_asset.occlusion_texture = material
            .occlusion_texture()
            .map(|texture| image_index_to_handle[texture.texture().source().index()].clone());
        material_asset.emissive_texture = material
            .emissive_texture()
            .map(|texture| image_index_to_handle[texture.texture().source().index()].clone());

        material_asset.material_data.has_base_color_texture =
            material_asset.base_color_texture.is_some();
        material_asset.material_data.has_metallic_roughness_texture =
            material_asset.metallic_roughness_texture.is_some();
        material_asset.material_data.has_normal_texture = material_asset.normal_texture.is_some();
        material_asset.material_data.has_occlusion_texture =
            material_asset.occlusion_texture.is_some();
        material_asset.material_data.has_emissive_texture =
            material_asset.emissive_texture.is_some();

        // pub base_color_texture: Option<Handle<ImageAsset>>,
        // // metalness in B, roughness in G
        // pub metallic_roughness_texture: Option<Handle<ImageAsset>>,
        // pub normal_texture: Option<Handle<ImageAsset>>,
        // pub occlusion_texture: Option<Handle<ImageAsset>>,
        // pub emissive_texture: Option<Handle<ImageAsset>>,

        let id = material
            .name()
            .map(|s| GltfObjectId::Name(s.to_string()))
            .unwrap_or(GltfObjectId::Index(material.index().unwrap()));

        let material_to_import = MaterialToImport {
            id,
            asset: material_asset,
        };

        // Verify that we iterate images in order so that our resulting assets are in order
        assert!(material.index().unwrap() == materials_to_import.len());
        log::debug!(
            "Importing Material name: {:?} index: {}",
            material.name(),
            material.index().unwrap(),
        );

        materials_to_import.push(material_to_import);
    }

    materials_to_import
}

//TODO: This feels kind of dumb..
fn convert_to_u16_indices(
    read_indices: gltf::mesh::util::ReadIndices
) -> Result<Vec<u16>, std::num::TryFromIntError> {
    let indices_u32: Vec<u32> = read_indices.into_u32().collect();
    let mut indices_u16: Vec<u16> = Vec::with_capacity(indices_u32.len());
    for index in indices_u32 {
        indices_u16.push(index.try_into()?);
    }

    Ok(indices_u16)
}

fn extract_meshes_to_import(
    state: &mut GltfImporterStateUnstable,
    doc: &gltf::Document,
    buffers: &Vec<GltfBufferData>,
    material_index_to_handle: &[Handle<GltfMaterialAsset>],
    material_instance_index_to_handle: &[Handle<MaterialInstanceAsset>],
) -> atelier_assets::importer::Result<(Vec<MeshToImport>, Vec<BufferToImport>)> {
    let mut meshes_to_import = Vec::with_capacity(doc.meshes().len());
    let mut buffers_to_import = Vec::with_capacity(doc.meshes().len() * 2);

    for mesh in doc.meshes() {
        let mut all_vertices = PushBuffer::new(16384);
        let mut all_indices = PushBuffer::new(16384);

        let mut mesh_parts: Vec<MeshPartData> = Vec::with_capacity(mesh.primitives().len());
        let mut bounding_aabb: Option<BoundingAabb> = None;

        //
        // Iterate all mesh parts, building a single vertex and index buffer. Each MeshPart will
        // hold offsets/lengths to their sections in the vertex/index buffers
        //
        for primitive in mesh.primitives() {
            let mesh_part = {
                let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|x| &**x));

                let positions = reader.read_positions();
                let normals = reader.read_normals();
                let tangents = reader.read_tangents();
                //let colors = reader.read_colors();
                let tex_coords = reader.read_tex_coords(0);
                let indices = reader.read_indices();

                if let (
                    Some(indices),
                    Some(positions),
                    Some(normals),
                    Some(tangents),
                    Some(tex_coords),
                ) = (indices, positions, normals, tangents, tex_coords)
                {
                    let part_indices = convert_to_u16_indices(indices);

                    if let Ok(part_indices) = part_indices {
                        //TODO: Consider computing binormal (bitangent) here
                        let positions: Vec<_> = positions.collect();
                        let normals: Vec<_> = normals.collect();
                        let tangents: Vec<_> = tangents.collect();
                        let tex_coords: Vec<_> = tex_coords.into_f32().collect();

                        let vertex_offset = all_vertices.len();
                        let indices_offset = all_indices.len();

                        for i in 0..positions.len() {
                            all_vertices.push(
                                &[MeshVertex {
                                    position: positions[i],
                                    normal: normals[i],
                                    tangent: tangents[i],
                                    tex_coord: tex_coords[i],
                                }],
                                1,
                            );

                            match &mut bounding_aabb {
                                Some(x) => x.expand(positions[i].into()),
                                None => {
                                    bounding_aabb = Some(BoundingAabb::new(positions[i].into()))
                                }
                            }
                        }

                        all_indices.push(&part_indices, 1);

                        let vertex_size = all_vertices.len() - vertex_offset;
                        let indices_size = all_indices.len() - indices_offset;

                        let (material, material_instance) = if let Some(material_index) =
                            primitive.material().index()
                        {
                            (
                                material_index_to_handle[material_index].clone(),
                                material_instance_index_to_handle[material_index].clone(),
                            )
                        } else {
                            return Err(atelier_assets::importer::Error::Boxed(Box::new(
                                GltfImportError::new("A mesh primitive did not have a material"),
                            )));
                        };

                        Some(MeshPartData {
                            material,
                            material_instance,
                            vertex_buffer_offset_in_bytes: vertex_offset as u32,
                            vertex_buffer_size_in_bytes: vertex_size as u32,
                            index_buffer_offset_in_bytes: indices_offset as u32,
                            index_buffer_size_in_bytes: indices_size as u32,
                        })
                    } else {
                        log::error!("indices must fit in u16");
                        None
                    }
                } else {
                    log::error!(
                        "Mesh primitives must specify indices, positions, normals, tangents, and tex_coords"
                    );
                    None
                }
            };

            if let Some(mesh_part) = mesh_part {
                mesh_parts.push(mesh_part);
            }
        }

        //
        // Vertex Buffer
        //
        let vertex_buffer_asset = BufferAssetData {
            data: all_vertices.into_data(),
        };

        let vertex_buffer_id = GltfObjectId::Index(buffers_to_import.len());
        let vertex_buffer_to_import = BufferToImport {
            asset: vertex_buffer_asset,
            id: vertex_buffer_id.clone(),
        };

        let vertex_buffer_uuid = *state
            .buffer_asset_uuids
            .entry(vertex_buffer_id)
            .or_insert_with(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));

        buffers_to_import.push(vertex_buffer_to_import);

        let vertex_buffer_handle =
            SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
                let load_handle = loader_info_provider
                    .get_load_handle(&AssetRef::Uuid(vertex_buffer_uuid))
                    .unwrap();
                Handle::<BufferAsset>::new(ref_op_sender.clone(), load_handle)
            });

        //
        // Index Buffer
        //
        let index_buffer_asset = BufferAssetData {
            data: all_indices.into_data(),
        };

        let index_buffer_id = GltfObjectId::Index(buffers_to_import.len());
        let index_buffer_to_import = BufferToImport {
            asset: index_buffer_asset,
            id: index_buffer_id.clone(),
        };

        let index_buffer_uuid = *state
            .buffer_asset_uuids
            .entry(index_buffer_id)
            .or_insert_with(|| AssetUuid(*uuid::Uuid::new_v4().as_bytes()));

        buffers_to_import.push(index_buffer_to_import);

        let index_buffer_handle =
            SerdeContext::with_active(|loader_info_provider, ref_op_sender| {
                let load_handle = loader_info_provider
                    .get_load_handle(&AssetRef::Uuid(index_buffer_uuid))
                    .unwrap();
                Handle::<BufferAsset>::new(ref_op_sender.clone(), load_handle)
            });

        if bounding_aabb.is_none() {
            bounding_aabb = Some(BoundingAabb::new(glam::Vec3::zero()));
        }

        let bounding_aabb = bounding_aabb.unwrap();
        let bounding_sphere = bounding_aabb.calculate_bounding_sphere();

        let asset = MeshAssetData {
            bounding_sphere,
            bounding_aabb,
            mesh_parts,
            vertex_buffer: vertex_buffer_handle,
            index_buffer: index_buffer_handle,
        };

        let mesh_id = mesh
            .name()
            .map(|s| GltfObjectId::Name(s.to_string()))
            .unwrap_or(GltfObjectId::Index(mesh.index()));

        let mesh_to_import = MeshToImport { id: mesh_id, asset };

        // Verify that we iterate meshes in order so that our resulting assets are in order
        assert!(mesh.index() == meshes_to_import.len());
        log::debug!(
            "Importing Mesh name: {:?} index: {} mesh_parts count: {}",
            mesh.name(),
            mesh.index(),
            mesh_to_import.asset.mesh_parts.len()
        );

        meshes_to_import.push(mesh_to_import);
    }

    Ok((meshes_to_import, buffers_to_import))
}

// let name = scene.name();
// for node in scene.nodes() {
//     for child in node.children() {
//         child.name();
//
//         //child.camera();
//         //child.light();
//         // - name
//         // - color
//         // - intensity
//         // - type (directional, point, spot)
//         // directional: emit -z, lm/m^2
//         // point: lm/sr
//         // spot: innerConeAngle, outerConeAngle, radians
//         // - Must be <= PI/2.0
//         // - Use outerConeAngle if there is no support for inner
//         // - PI/4 default
//         //
//         // - range
//
//
//         //child.mesh();
//     }
// }
fn add_nodes_to_world(
    mesh_index_to_handle: &[Handle<MeshAsset>],
    world: &mut World,
    node: &gltf::Node,
    parent_transform: glam::Mat4,
) {
    let local_to_world =
        parent_transform * glam::Mat4::from_cols_array_2d(&node.transform().matrix());
    if let Some(mesh) = node.mesh() {
        //let transform_component = TransformComponentDef::from_matrix(local_to_world);
        let mesh_handle = mesh_index_to_handle[mesh.index()].clone();
        let mesh_component = MeshComponentDef {
            mesh: Some(mesh_handle.into()),
        };

        // Temporary
        let transform_component = {
            let m = node.transform().matrix();
            let m = glam::Mat4::from_cols_array_2d(&m);
            let tformed = parent_transform * m;
            TransformComponent { transform: tformed }
        };

        let components = vec![(transform_component, mesh_component)];
        let e = world.extend(components)[0];

        log::info!("Added mesh {:?}", e);
        if let Some(name) = node.name() {
            log::info!("  name: {}", name);
            world
                .entry(e)
                .unwrap()
                .add_component(EditorMetadataComponent {
                    name: name.to_string(),
                });
        };
    }

    if let Some(light) = node.light() {
        let transform_component = TransformComponentDef::from_matrix(local_to_world);
        let intensity = light.intensity();
        //TODO: Better default for range
        let range = light.range().unwrap_or(f32::MAX);
        let color = light.color().into();

        let entity = match light.kind() {
            gltf::khr_lights_punctual::Kind::Directional => {
                let light_component = DirectionalLightComponent {
                    color,
                    intensity,
                    direction: glam::Vec3::new(0.0, 0.0, -1.0).into(), // per spec, directional lights point -z
                };

                let components = vec![(transform_component, light_component)];
                world.extend(components)[0]
            }
            gltf::khr_lights_punctual::Kind::Point => {
                let light_component = PointLightComponent {
                    color,
                    intensity,
                    range,
                };

                let components = vec![(transform_component, light_component)];
                world.extend(components)[0]
            }
            gltf::khr_lights_punctual::Kind::Spot {
                outer_cone_angle,
                inner_cone_angle,
            } => {
                //TODO: Support inner angle. Per spec, implementations should use outer if they only
                // accept a single value
                std::mem::forget(inner_cone_angle);

                let light_component = SpotLightComponent {
                    color,
                    intensity,
                    range,
                    direction: glam::Vec3::new(0.0, 0.0, -1.0).into(), // per spec, directional lights point -z,
                    spotlight_half_angle: outer_cone_angle,
                };

                let components = vec![(transform_component, light_component)];
                world.extend(components)[0]
            }
        };
        log::info!("Added mesh {:?}", entity);

        if let Some(name) = node.name() {
            log::info!("  name: {}", name);
            world
                .entry(entity)
                .unwrap()
                .add_component(EditorMetadataComponent {
                    name: name.to_string(),
                });
        };
    }

    if let Some(camera) = node.camera() {
        //GLTF:
        // The camera is defined such that the local +X axis is to the right,
        // the lens looks towards the local -Z axis, and
        // the top of the camera is aligned with the local +Y axis
        //BLENDER:
        // X = Left/Right   (+X = Left)
        // Y = Front/back   (+Y = Back)
        // Z = Top/Bottom   (+Z = Top
        match camera.projection() {
            Projection::Orthographic(proj) => {
                proj.xmag();
                proj.ymag();
                proj.zfar();
                proj.znear();
            }
            Projection::Perspective(proj) => {
                proj.aspect_ratio();
                proj.yfov();
                proj.zfar();
                proj.znear();
            }
        }
    }

    for child in node.children() {
        add_nodes_to_world(mesh_index_to_handle, world, &child, local_to_world);
    }
}

fn extract_prefabs_to_import(
    doc: &gltf::Document,
    mesh_index_to_handle: &[Handle<MeshAsset>],
    prefabs_uuids: &mut FnvHashMap<GltfObjectId, AssetUuid>,
) -> Vec<PrefabToImport> {
    let mut prefabs_to_import = Vec::with_capacity(doc.scenes().len());

    for scene in doc.scenes() {
        // Create an empty world for the scene
        let mut world = World::default();

        // Descend the node tree recursively, adding things to the world
        for node in scene.nodes() {
            let transform = glam::Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2);
            //let transform = glam::Mat4::identity();
            add_nodes_to_world(mesh_index_to_handle, &mut world, &node, transform);
        }

        // Turn the world into a prefab
        let mut prefab = Prefab::new(world);

        // Use the scene name, or index if unnamed, to create a "stable" ID within this gltf file
        let scene_id = scene
            .name()
            .map(|s| GltfObjectId::Name(s.to_string()))
            .unwrap_or(GltfObjectId::Index(scene.index()));

        // If we have exported a scene with a matching ID previous, use the same uuid as last time
        if let Some(previous_uuid) = prefabs_uuids.get(&scene_id) {
            println!("Found previous ID");
            prefab.prefab_meta.id = previous_uuid.0;
        } else {
            println!("Inserting new ID");
            prefabs_uuids.insert(scene_id.clone(), AssetUuid(prefab.prefab_id()));
        };

        prefabs_to_import.push(PrefabToImport {
            id: scene_id,
            asset: PrefabAsset { prefab },
        });
    }

    prefabs_to_import
}
