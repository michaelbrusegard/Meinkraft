use crate::input::InputManager;
use crate::state::GameState;
use crate::systems::{ChunkLoadingSystem, ChunkMeshingSystem, InputSystem, RenderSystem};

pub struct SystemScheduler {
    input_system: InputSystem,
    chunk_loading_system: ChunkLoadingSystem,
    chunk_meshing_system: ChunkMeshingSystem,
    render_system: RenderSystem,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            input_system: InputSystem::new(),
            chunk_loading_system: ChunkLoadingSystem::new(),
            chunk_meshing_system: ChunkMeshingSystem::new(),
            render_system: RenderSystem::new(),
        }
    }

    pub fn update_input(&mut self, game_state: &mut GameState, input_manager: &InputManager) {
        self.input_system.update(
            &game_state.config,
            &mut game_state.world,
            &game_state.input_state,
            &mut game_state.camera,
            input_manager,
        );
    }

    pub fn process_updates_and_requests(&mut self, game_state: &mut GameState) {
        self.chunk_loading_system.update(game_state);

        self.chunk_meshing_system
            .process_mesh_results_and_requests(game_state);
    }

    pub fn render(&self, game_state: &mut GameState) {
        self.render_system.render(
            &game_state.world,
            &mut game_state.camera,
            &game_state.renderer,
            &game_state.shader_program,
            &game_state.texture_manager,
            &game_state.mesh_registry,
        );
    }
}
