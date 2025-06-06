use crate::components::{ChunkCoord, ChunkData};
use crate::resources::{Config, MeshGenerator, WorldGenerator};
use crate::state::{MeshRequestData, MeshResultData};
use bincode::config::{standard, Configuration};
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap as StdHashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Error as IoError, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

const BINCODE_CONFIG: Configuration = standard();

#[derive(Debug, Clone, Copy)]
pub enum LoadRequest {
    LoadOrGenerate(ChunkCoord),
    LoadFromCache(ChunkCoord),
}

pub type LoadResult = (ChunkCoord, Option<ChunkData>);

pub struct WorkerResources {
    pub world_generator: Arc<WorldGenerator>,
    pub mesh_generator: Arc<MeshGenerator>,
    pub texture_manager_layers: Arc<StdHashMap<String, f32>>,
    pub chunk_cache: ChunkCache,
    pub config: Config,
}

pub struct WorkerChannels {
    pub gen_request_rx: Receiver<LoadRequest>,
    pub mesh_request_rx: Receiver<MeshRequestData>,
    pub gen_result_tx: Sender<LoadResult>,
    pub mesh_result_tx: Sender<MeshResultData>,
}

#[derive(Clone)]
pub struct ChunkCache {
    cache_dir: PathBuf,
}

impl ChunkCache {
    pub fn new(world_name: &str) -> Result<Self, String> {
        let cache_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .join("cache")
            .join(world_name)
            .join("chunks");

        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory {:?}: {}", cache_dir, e))?;

        Ok(Self { cache_dir })
    }

    fn get_chunk_path(&self, coord: ChunkCoord) -> PathBuf {
        self.cache_dir
            .join(format!("{}_{}_{}.chunk", coord.0, coord.1, coord.2))
    }

    pub fn save_chunk(&self, coord: ChunkCoord, chunk_data: &ChunkData) -> Result<(), IoError> {
        let path = self.get_chunk_path(coord);
        match File::create(&path) {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                bincode::serde::encode_into_std_write(chunk_data, &mut writer, BINCODE_CONFIG)
                    .map(|_| ())
                    .map_err(|e| {
                        IoError::new(ErrorKind::Other, format!("Bincode encode error: {}", e))
                    })
            }
            Err(e) => {
                eprintln!("Failed to create/truncate file {:?}: {}", path, e);
                Err(e)
            }
        }
    }

    pub fn load_chunk(&self, coord: ChunkCoord) -> Result<Option<ChunkData>, IoError> {
        let path = self.get_chunk_path(coord);
        if !path.exists() {
            return Ok(None);
        }

        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let chunk_data: ChunkData =
            bincode::serde::decode_from_std_read(&mut reader, BINCODE_CONFIG).map_err(|e| {
                IoError::new(
                    ErrorKind::InvalidData,
                    format!("Bincode decode error at {:?}: {}", path, e),
                )
            })?;
        Ok(Some(chunk_data))
    }

    pub fn delete_chunk(&self, coord: ChunkCoord) -> Result<(), IoError> {
        let path = self.get_chunk_path(coord);
        if path.exists() {
            fs::remove_file(path)
        } else {
            Ok(())
        }
    }
}

pub type NeighborData = Box<[Option<ChunkData>; 6]>;

#[allow(dead_code)]
pub struct WorkerPool {
    world_generator: Arc<WorldGenerator>,
    mesh_generator: Arc<MeshGenerator>,
    texture_manager_layers: Arc<StdHashMap<String, f32>>,
    chunk_cache: ChunkCache,
    config: Config,
    gen_request_rx: Receiver<LoadRequest>,
    mesh_request_rx: Receiver<MeshRequestData>,
    gen_result_tx: Sender<LoadResult>,
    mesh_result_tx: Sender<MeshResultData>,
    shutdown_tx: Sender<()>,
    worker_handles: Vec<thread::JoinHandle<()>>,
}

impl WorkerPool {
    pub fn new(resources: WorkerResources, channels: WorkerChannels) -> Self {
        let num_threads = num_cpus::get().saturating_sub(1).max(1);
        let mut worker_handles = Vec::with_capacity(num_threads);
        let (shutdown_tx, shutdown_rx) = crossbeam_channel::bounded::<()>(num_threads);
        let wg = Arc::clone(&resources.world_generator);
        let mg = Arc::clone(&resources.mesh_generator);
        let tm_layers = Arc::clone(&resources.texture_manager_layers);
        let cache = resources.chunk_cache.clone();
        let config = resources.config.clone();
        let gen_rx = channels.gen_request_rx.clone();
        let mesh_rx = channels.mesh_request_rx.clone();
        let gen_tx = channels.gen_result_tx.clone();
        let mesh_tx = channels.mesh_result_tx.clone();

        for i in 0..num_threads {
            let wg_clone = Arc::clone(&wg);
            let mg_clone = Arc::clone(&mg);
            let tm_layers_clone = Arc::clone(&tm_layers);
            let cache_clone = cache.clone();
            let config_clone = config.clone();
            let gen_rx_clone = gen_rx.clone();
            let mesh_rx_clone = mesh_rx.clone();
            let gen_tx_clone = gen_tx.clone();
            let mesh_tx_clone = mesh_tx.clone();
            let shutdown_rx_clone = shutdown_rx.clone();

            let handle = thread::Builder::new()
                .name(format!("worker-{}", i))
                .spawn(move || {
                    let wg = wg_clone;
                    let mg = mg_clone;
                    let tm_layers = tm_layers_clone;
                    let cache = cache_clone;
                    let config = config_clone;
                    let gen_rx = gen_rx_clone;
                    let mesh_rx = mesh_rx_clone;
                    let gen_tx = gen_tx_clone;
                    let mesh_tx = mesh_tx_clone;

                    loop {
                        crossbeam_channel::select! {
                            recv(gen_rx) -> msg => match msg {
                                Ok(request) => {
                                    let (coord, should_generate) = match request {
                                        LoadRequest::LoadOrGenerate(c) => (c, true),
                                        LoadRequest::LoadFromCache(c) => (c, false),
                                    };

                                    match cache.load_chunk(coord) {
                                        Ok(Some(data)) => {
                                            if gen_tx.send((coord, Some(data))).is_err() { break; }
                                        },
                                        Ok(None) => {
                                            if should_generate {
                                                let data = wg.generate_chunk_data(coord);
                                                if let Err(e) = cache.save_chunk(coord, &data) {
                                                     eprintln!("Worker {}: Error saving newly generated chunk {:?}: {}", i, coord, e);
                                                }
                                                if gen_tx.send((coord, Some(data))).is_err() { break; }
                                            } else if gen_tx.send((coord, None)).is_err() { break; }
                                        },
                                        Err(e) => {
                                            eprintln!("Worker {}: Error loading chunk {:?} from cache: {}", i, coord, e);
                                            if should_generate {
                                                let data = wg.generate_chunk_data(coord);
                                                 if let Err(e_save) = cache.save_chunk(coord, &data) {
                                                     eprintln!("Worker {}: Error saving chunk {:?} after load fail: {}", i, coord, e_save);
                                                }
                                                if gen_tx.send((coord, Some(data))).is_err() { break; }
                                            } else if gen_tx.send((coord, None)).is_err() { break; }
                                        }
                                    }
                                },
                                Err(_) => { break; }
                            },
                            recv(mesh_rx) -> msg => match msg {
                                Ok((entity, coord, chunk_data, neighbors, lod)) => {
                                    let mesh_result = mg.generate_chunk_mesh(
                                        coord,
                                        &chunk_data,
                                        &neighbors,
                                        &tm_layers,
                                        lod,
                                        &config,
                                    );
                                    if mesh_tx.send((entity, coord, mesh_result, lod)).is_err() {
                                        break;
                                    }
                                },
                                Err(_) => { break; }
                            },
                            recv(shutdown_rx_clone) -> _ => {
                                break;
                            }
                        }
                    }
                })
                .expect("Failed to spawn worker thread");

            worker_handles.push(handle);
        }

        Self {
            world_generator: resources.world_generator,
            mesh_generator: resources.mesh_generator,
            texture_manager_layers: resources.texture_manager_layers,
            chunk_cache: resources.chunk_cache,
            config: resources.config,
            gen_request_rx: channels.gen_request_rx,
            mesh_request_rx: channels.mesh_request_rx,
            gen_result_tx: channels.gen_result_tx,
            mesh_result_tx: channels.mesh_result_tx,
            shutdown_tx,
            worker_handles,
        }
    }

    pub fn shutdown(self) {
        drop(self.gen_request_rx);
        drop(self.mesh_request_rx);
        for _ in 0..self.worker_handles.len() {
            let _ = self.shutdown_tx.send(());
        }
        drop(self.shutdown_tx);

        for (i, handle) in self.worker_handles.into_iter().enumerate() {
            if let Err(e) = handle.join() {
                eprintln!("WorkerPool: Worker thread {} panicked: {:?}", i, e);
            }
        }
    }
}
