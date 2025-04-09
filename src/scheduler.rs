use crate::input::InputManager;
use crate::state::GameState;
use crate::systems::{ChunkMeshingSystem, InputSystem, RenderSystem};

pub struct SystemScheduler {
    input_system: InputSystem,
    chunk_meshing_system: ChunkMeshingSystem,
    render_system: RenderSystem,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            input_system: InputSystem::new(),
            chunk_meshing_system: ChunkMeshingSystem::new(),
            render_system: RenderSystem::new(),
        }
    }

    pub fn update(&mut self, game_state: &mut GameState, input_manager: &InputManager) {
        self.input_system.update(
            &game_state.config,
            &mut game_state.world,
            &game_state.input_state,
            &mut game_state.camera,
            input_manager,
        );

        self.chunk_meshing_system.update(game_state);
    }

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
