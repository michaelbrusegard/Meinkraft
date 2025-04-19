use crate::components::{ChunkCoord, ChunkData};
use crate::resources::{Mesh, MeshGenerator, TextureUVs, WorldGenerator};
use bincode::config::{standard, Configuration};
use crossbeam_channel::{Receiver, Sender};
use hecs::Entity;
use std::collections::HashMap as StdHashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Error as IoError, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

const BINCODE_CONFIG: Configuration = standard();

pub struct WorkerResources {
    pub world_generator: Arc<WorldGenerator>,
    pub mesh_generator: Arc<MeshGenerator>,
    pub texture_manager_uvs: Arc<StdHashMap<String, TextureUVs>>,
    pub chunk_cache: ChunkCache,
}

pub struct WorkerChannels {
    pub gen_request_rx: Receiver<ChunkCoord>,
    pub mesh_request_rx: Receiver<(Entity, ChunkCoord, ChunkData, NeighborData)>,
    pub gen_result_tx: Sender<(ChunkCoord, ChunkData)>,
    pub mesh_result_tx: Sender<(Entity, ChunkCoord, Option<Mesh>)>,
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
            .join("worlds")
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
                    .map(|_| ()) // Map Ok(usize) to Ok(())
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
    texture_manager_uvs: Arc<StdHashMap<String, TextureUVs>>,
    chunk_cache: ChunkCache,
    gen_request_rx: Receiver<ChunkCoord>,
    mesh_request_rx: Receiver<(Entity, ChunkCoord, ChunkData, NeighborData)>,
    gen_result_tx: Sender<(ChunkCoord, ChunkData)>,
    mesh_result_tx: Sender<(Entity, ChunkCoord, Option<Mesh>)>,
    shutdown_tx: Sender<()>,
    worker_handles: Vec<thread::JoinHandle<()>>,
}

impl WorkerPool {
    pub fn new(resources: WorkerResources, channels: WorkerChannels) -> Self {
        let num_threads = num_cpus::get().saturating_sub(1).max(1);
        let mut worker_handles = Vec::with_capacity(num_threads);
        let (shutdown_tx, shutdown_rx) = crossbeam_channel::bounded::<()>(num_threads);

        println!("Spawning {} worker threads...", num_threads);

        let wg = Arc::clone(&resources.world_generator);
        let mg = Arc::clone(&resources.mesh_generator);
        let tm_uvs = Arc::clone(&resources.texture_manager_uvs);
        let cache = resources.chunk_cache.clone();

        let gen_rx = channels.gen_request_rx.clone();
        let mesh_rx = channels.mesh_request_rx.clone();
        let gen_tx = channels.gen_result_tx.clone();
        let mesh_tx = channels.mesh_result_tx.clone();

        for i in 0..num_threads {
            let wg_clone = Arc::clone(&wg);
            let mg_clone = Arc::clone(&mg);
            let tm_uvs_clone = Arc::clone(&tm_uvs);
            let cache_clone = cache.clone();
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
                    let tm_uvs = tm_uvs_clone;
                    let cache = cache_clone;
                    let gen_rx = gen_rx_clone;
                    let mesh_rx = mesh_rx_clone;
                    let gen_tx = gen_tx_clone;
                    let mesh_tx = mesh_tx_clone;

                    println!("Worker thread {} started.", i);
                    loop {
                        crossbeam_channel::select! {
                            recv(gen_rx) -> msg => match msg {
                                Ok(coord) => {
                                    match cache.load_chunk(coord) {
                                        Ok(Some(data)) => {
                                            if gen_tx.send((coord, data)).is_err() { break; }
                                        },
                                        Ok(None) => {
                                            let data = wg.generate_chunk_data(coord);
                                            if let Err(e) = cache.save_chunk(coord, &data) {
                                                 eprintln!("Worker {}: Error saving newly generated chunk {:?}: {}", i, coord, e);
                                            }
                                            if gen_tx.send((coord, data)).is_err() { break; }
                                        },
                                        Err(e) => {
                                            eprintln!("Worker {}: Error loading chunk {:?} from cache: {}", i, coord, e);
                                            let data = wg.generate_chunk_data(coord);
                                             if let Err(e_save) = cache.save_chunk(coord, &data) {
                                                 eprintln!("Worker {}: Error saving chunk {:?} after load fail: {}", i, coord, e_save);
                                            }
                                            if gen_tx.send((coord, data)).is_err() { break; }
                                        }
                                    }
                                },
                                Err(_) => { break; }
                            },
                            recv(mesh_rx) -> msg => match msg {
                                Ok((entity, coord, chunk_data, neighbors)) => {
                                    let mesh_result = mg.generate_chunk_mesh(
                                        coord,
                                        &chunk_data,
                                        &neighbors,
                                        &tm_uvs,
                                    );
                                    if mesh_tx.send((entity, coord, mesh_result)).is_err() {
                                        break;
                                    }
                                },
                                Err(_) => { break; }
                            },
                            recv(shutdown_rx_clone) -> _ => {
                                println!("Worker {}: Received shutdown signal, exiting.", i);
                                break;
                            }
                        }
                    }
                    println!("Worker thread {} finished.", i);
                })
                .expect("Failed to spawn worker thread");

            worker_handles.push(handle);
        }

        Self {
            world_generator: resources.world_generator,
            mesh_generator: resources.mesh_generator,
            texture_manager_uvs: resources.texture_manager_uvs,
            chunk_cache: resources.chunk_cache,
            gen_request_rx: channels.gen_request_rx,
            mesh_request_rx: channels.mesh_request_rx,
            gen_result_tx: channels.gen_result_tx,
            mesh_result_tx: channels.mesh_result_tx,
            shutdown_tx,
            worker_handles,
        }
    }

    pub fn shutdown(self) {
        println!("WorkerPool: Sending shutdown signal...");
        drop(self.shutdown_tx);

        println!("WorkerPool: Joining worker threads...");
        for (i, handle) in self.worker_handles.into_iter().enumerate() {
            if let Err(e) = handle.join() {
                eprintln!("WorkerPool: Worker thread {} panicked: {:?}", i, e);
            }
        }
        println!("Worker pool shut down completely.");
    }
}
