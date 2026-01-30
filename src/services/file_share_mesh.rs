// File transfer service wrapper
// Provides a simplified interface to the mesh file sharing implementation

mod mesh_impl;

use std::path::Path;
pub use mesh_impl::{MeshFileShare, SharedFile, FileChunk};

pub struct FileTransfer {
    mesh: MeshFileShare,
}

impl FileTransfer {
    pub async fn new() -> Self {
        Self {
            mesh: MeshFileShare::new(),
        }
    }
    
    pub async fn share_file(&self, path: &Path) -> Result<String, anyhow::Error> {
        // Use a default peer ID for now
        let peer_id = "local".to_string();
        self.mesh.share_file(path, peer_id).await
    }
    
    pub async fn download_file(&self, file_hash: &str, output_path: &Path) -> Result<(), anyhow::Error> {
        self.mesh.download_file(file_hash, output_path).await
    }
    
    pub fn get_shared_files(&self) -> Vec<SharedFile> {
        self.mesh.shared_files
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}