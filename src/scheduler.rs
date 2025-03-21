use crate::state::GameState;
use crate::systems::{InitSystem, InputSystem, RenderSystem};

pub struct SystemScheduler {
    init_system: InitSystem,
    input_system: InputSystem,
    render_system: RenderSystem,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            init_system: InitSystem::new(),
            input_system: InputSystem::new(),
            render_system: RenderSystem::new(),
        }
    }

    pub fn initialize(&mut self, game_state: &mut GameState) {
        self.init_system.initialize(
            &mut game_state.world,
            &mut game_state.mesh_registry,
            &mut game_state.renderer,
        );
    }

    pub fn update(&mut self, game_state: &mut GameState) {
        self.input_system.update(
            &game_state.config,
            &mut game_state.world,
            &game_state.input_state,
            &mut game_state.camera,
        );
    }

    pub fn render(&self, game_state: &GameState) {
        self.render_system.render(
            &game_state.world,
            &game_state.camera,
            &game_state.renderer,
            &game_state.shader_program,
        );
    }
}
