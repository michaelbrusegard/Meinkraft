// scheduler.rs
use crate::input::InputManager;
use crate::state::GameState;
use crate::systems::{ChunkLoadingSystem, ChunkMeshingSystem, InputSystem, RenderSystem};

pub struct SystemScheduler {
    input_system: InputSystem,
    chunk_loading_system: ChunkLoadingSystem,
    chunk_meshing_system: ChunkMeshingSystem, // Handles meshing requests/results
    render_system: RenderSystem,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            input_system: InputSystem::new(),
            chunk_loading_system: ChunkLoadingSystem::new(),
            chunk_meshing_system: ChunkMeshingSystem::new(), // Initialize stateful system
            render_system: RenderSystem::new(),
        }
    }

    // Stage 1: Handle input, update camera, potentially mark chunks modified/dirty
    pub fn update_input(&mut self, game_state: &mut GameState, input_manager: &InputManager) {
        self.input_system.update(
            &game_state.config,
            &mut game_state.world, // Needs mut world for block modification
            &game_state.input_state,
            &mut game_state.camera,
            input_manager,
        );
        // InputSystem should add ChunkModified and ChunkDirty tags when blocks change
    }

    // Stage 2: Process generation results, request new chunks, process mesh results, request new meshes
    pub fn process_updates_and_requests(&mut self, game_state: &mut GameState) {
        // Process generation results and request loading/unloading
        // (This also adds ChunkDirty tags when chunks load)
        self.chunk_loading_system.update(game_state);

        // Process mesh results from workers and request meshing for dirty chunks
        // (This consumes ChunkDirty tags after sending requests)
        self.chunk_meshing_system
            .process_mesh_results_and_requests(game_state);
    }

    // Stage 3: Render the world based on current components
    pub fn render(&self, game_state: &GameState) {
        self.render_system.render(
            &game_state.world,
            &game_state.camera,
            &game_state.renderer,
            &game_state.shader_program,
            &game_state.texture_manager,
            &game_state.mesh_registry,
        );
    }
}
