use crate::state::GameState;
use crate::systems::{InputSystem, RenderSystem};

pub struct SystemScheduler {
    input_system: InputSystem,
    render_system: RenderSystem,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            input_system: InputSystem::new(),
            render_system: RenderSystem::new(),
        }
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
            &game_state.texture_manager,
        );
    }
}
