use renderer::assets::AssetLookup;
use crate::assets::gltf::MeshAsset;

#[derive(Debug)]
pub struct GameAssetMetrics {
    pub mesh_count: usize,
}

//
// Lookups by asset for loaded asset state
//
#[derive(Default)]
pub struct GameAssetLookupSet {
    pub meshes: AssetLookup<MeshAsset>,
}

impl GameAssetLookupSet {
    pub fn metrics(&self) -> GameAssetMetrics {
        GameAssetMetrics {
            mesh_count: self.meshes.len(),
        }
    }

    pub fn destroy(&mut self) {
        self.meshes.destroy();
    }
}
