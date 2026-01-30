use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use dashmap::DashMap;
use blake3::Hasher;
use serde::{Serialize, Deserialize};
use anyhow::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferProgress {
    pub file_name: String,
    pub file_hash: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub status: TransferStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

pub struct FileTransfer {
    shared_files: Arc<DashMap<String, SharedFile>>,
    file_chunks: Arc<DashMap<String, FileChunk>>,
    downloading_files: Arc<RwLock<HashMap<String, DownloadingFile>>>,
    peers_with_files: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    active_transfers: Arc<RwLock<HashMap<String, TransferProgress>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SharedFile {
    pub hash: String,
    pub name: String,
    pub size: u64,
    pub chunks: Vec<String>,
    pub chunk_size: u64,
    pub total_chunks: usize,
    pub peer_id: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct FileChunk {
    pub chunk_hash: String,
    pub data: Vec<u8>,
    pub index: usize,
    pub file_hash: String,
}

#[derive(Debug)]
pub struct DownloadingFile {
    pub file_hash: String,
    pub chunks_received: HashSet<usize>,
    pub chunks_expected: usize,
    pub peers: HashSet<String>,
    pub output_path: PathBuf,
    pub bytes_received: u64,
}

impl FileTransfer {
    pub async fn new() -> Self {
        FileTransfer {
            shared_files: Arc::new(DashMap::new()),
            file_chunks: Arc::new(DashMap::new()),
            downloading_files: Arc::new(RwLock::new(HashMap::new())),
            peers_with_files: Arc::new(RwLock::new(HashMap::new())),
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn share_file(&self, path: &Path, peer_id: String) -> Result<String, Error> {
        // Read file and calculate hash
        let data = tokio::fs::read(path).await?;
        let hash = Self::calculate_file_hash(&data);
        
        // Split into chunks (1MB each)
        let chunk_size = 1024 * 1024; // 1MB
        let chunks: Vec<Vec<u8>> = data.chunks(chunk_size).map(|c| c.to_vec()).collect();
        
        // Create shared file record
        let shared_file = SharedFile {
            hash: hash.clone(),
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            size: data.len() as u64,
            chunks: chunks.iter().enumerate().map(|(i, chunk)| {
                Self::calculate_chunk_hash(i, chunk)
            }).collect(),
            chunk_size: chunk_size as u64,
            total_chunks: chunks.len(),
            peer_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Store chunks
        for (i, chunk) in chunks.into_iter().enumerate() {
            let chunk_hash = Self::calculate_chunk_hash(i, &chunk);
            self.file_chunks.insert(chunk_hash.clone(), FileChunk {
                chunk_hash: chunk_hash.clone(),
                data: chunk,
                index: i,
                file_hash: hash.clone(),
            });
        }
        
        // Announce file to network
        self.announce_file(&shared_file).await?;
        
        // Create transfer progress entry
        let progress = TransferProgress {
            file_name: shared_file.name.clone(),
            file_hash: hash.clone(),
            bytes_transferred: 0,
            total_bytes: shared_file.size,
            percentage: 0.0,
            status: TransferStatus::Completed,
        };
        
        self.active_transfers.write().await.insert(hash.clone(), progress);
        
        Ok(hash)
    }
    
    pub async fn download_file(&self, file_hash: &str, output_path: &Path) -> Result<(), Error> {
        // Get file info from DHT or direct from peers
        if let Some(file) = self.shared_files.get(file_hash) {
            let downloading = DownloadingFile {
                file_hash: file_hash.to_string(),
                chunks_received: HashSet::new(),
                chunks_expected: file.total_chunks,
                peers: HashSet::new(),
                output_path: output_path.to_path_buf(),
                bytes_received: 0,
            };
            
            self.downloading_files.write().await.insert(file_hash.to_string(), downloading);
            
            // Create progress entry
            let progress = TransferProgress {
                file_name: file.name.clone(),
                file_hash: file_hash.to_string(),
                bytes_transferred: 0,
                total_bytes: file.size,
                percentage: 0.0,
                status: TransferStatus::InProgress,
            };
            
            self.active_transfers.write().await.insert(file_hash.to_string(), progress);
            
            // Request chunks from multiple peers
            self.request_chunks(file_hash).await?;
        }
        
        Ok(())
    }
    
    pub async fn get_transfer_progress(&self) -> Vec<TransferProgress> {
        self.active_transfers.read().await.values().cloned().collect()
    }
    
    pub async fn list_files_in_directory(&self, path: &str) -> Result<Vec<String>, Error> {
        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    files.push(name.to_string_lossy().to_string());
                }
            }
        }
        
        Ok(files)
    }
    
    pub async fn send_file_to_device(&self, device_ip: &str, file_path: &str) -> Result<(), Error> {
        // This would use the P2P transport to send file
        // For now, we'll simulate the transfer
        let path = Path::new(file_path);
        let peer_id = device_ip.to_string();
        
        let file_hash = self.share_file(path, peer_id).await?;
        
        // Simulate download on the other side
        let output_path = PathBuf::from(format!("/tmp/{}", path.file_name().unwrap().to_string_lossy()));
        self.download_file(&file_hash, &output_path).await?;
        
        Ok(())
    }
    
    async fn request_chunks(&self, file_hash: &str) -> Result<(), Error> {
        if let Some(file) = self.shared_files.get(file_hash) {
            // Get peers that have this file
            let peers = self.peers_with_files.read().await;
            if let Some(file_peers) = peers.get(file_hash) {
                // Request chunks from different peers (load balancing)
                for (chunk_index, chunk_hash) in file.chunks.iter().enumerate() {
                    // Find peer with this chunk
                    for peer_id in file_peers {
                        // Send chunk request
                        self.request_chunk_from_peer(peer_id, chunk_hash, chunk_index).await?;
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn request_chunk_from_peer(&self, peer_id: &str, chunk_hash: &str, chunk_index: usize) -> Result<(), Error> {
        // This would use our P2P transport
        // For now, we'll simulate receiving the chunk
        if let Some(chunk) = self.file_chunks.get(chunk_hash) {
            self.handle_chunk_received(chunk_hash, chunk_index, chunk.data.clone()).await?;
        }
        Ok(())
    }
    
    pub async fn handle_chunk_request(&self, chunk_hash: &str, from: String) -> Result<(), Error> {
        if let Some(chunk) = self.file_chunks.get(chunk_hash) {
            // Send chunk back to requester
            self.send_chunk_to_peer(from, chunk.value().clone()).await?;
        }
        Ok(())
    }
    
    async fn send_chunk_to_peer(&self, peer_id: String, chunk: FileChunk) -> Result<(), Error> {
        // This would use our P2P transport
        // For simulation, we'll store it in the receiving peer's chunks
        Ok(())
    }
    
    async fn handle_chunk_received(&self, chunk_hash: &str, chunk_index: usize, data: Vec<u8>) -> Result<(), Error> {
        // Update downloading file progress
        let mut downloading_files = self.downloading_files.write().await;
        
        // Find the file that this chunk belongs to
        for mut entry in downloading_files.iter_mut() {
            let downloading = entry.value_mut();
            if downloading.chunks_received.contains(&chunk_index) {
                continue;
            }
            
            // Check if this chunk belongs to this file
            if let Some(file) = self.shared_files.get(&downloading.file_hash) {
                if chunk_index < file.chunks.len() && file.chunks[chunk_index] == chunk_hash {
                    downloading.chunks_received.insert(chunk_index);
                    downloading.bytes_received += data.len() as u64;
                    
                    // Update progress
                    if let Some(mut progress) = self.active_transfers.write().await.get_mut(&downloading.file_hash) {
                        progress.bytes_transferred = downloading.bytes_received;
                        progress.percentage = (downloading.bytes_received as f64 / file.size as f64) * 100.0;
                        
                        if downloading.chunks_received.len() == downloading.chunks_expected {
                            progress.status = TransferStatus::Completed;
                            self.assemble_file(downloading).await?;
                        }
                    }
                    
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn assemble_file(&self, downloading: &DownloadingFile) -> Result<(), Error> {
        // Assemble all chunks into the final file
        let mut file_data = Vec::new();
        
        for i in 0..downloading.chunks_expected {
            if let Some(chunk) = self.file_chunks.get(&downloading.file_hash) {
                file_data.extend_from_slice(&chunk.data);
            }
        }
        
        // Write to output path
        tokio::fs::write(&downloading.output_path, file_data).await?;
        
        Ok(())
    }
    
    async fn announce_file(&self, file: &SharedFile) -> Result<(), Error> {
        // Store in local registry
        self.shared_files.insert(file.hash.clone(), file.clone());
        
        // Announce to connected peers
        // This would use our P2P network broadcast
        Ok(())
    }
    
    fn calculate_file_hash(data: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hex::encode(hasher.finalize().as_bytes())
    }
    
    fn calculate_chunk_hash(index: usize, data: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(&index.to_le_bytes());
        hasher.update(data);
        hex::encode(hasher.finalize().as_bytes())
    }
}