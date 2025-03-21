use crate::resources::InputState;
use crate::systems::{InputSystem, RenderSystem};
use glutin::config::ConfigTemplateBuilder;
use glutin_winit::DisplayBuilder;
use std::error::Error;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::game_state::GameState;
use crate::resources::Config;
use crate::window_manager::WindowManager;

pub struct App {
    window_manager: WindowManager,
    game_state: Option<GameState>,
    render_system: RenderSystem,
    input_state: InputState,
    input_system: InputSystem,
    pub exit_state: Result<(), Box<dyn Error>>,
}

impl App {
    pub fn new(template: ConfigTemplateBuilder, display_builder: DisplayBuilder) -> Self {
        let config = Config::new();
        Self {
            window_manager: WindowManager::new(template, display_builder),
            exit_state: Ok(()),
            game_state: None,
            render_system: RenderSystem::new(),
            input_state: InputState::new(),
            input_system: InputSystem::new(config),
        }
    }

    fn initialize_game(&mut self) {
        let gl = self.window_manager.create_gl();
        let (width, height) = self.window_manager.get_dimensions().unwrap_or((800, 600));

        self.game_state = Some(GameState::new(gl, width, height));
        self.window_manager.initialize_window();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.window_manager.resume(event_loop) {
            Ok(()) => {
                if self.game_state.is_none() {
                    self.initialize_game();
                }
            }
            Err(err) => {
                self.exit_state = Err(err);
                event_loop.exit();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.window_manager.suspend();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                if let Some(game_state) = &mut self.game_state {
                    self.window_manager.resize(size.width, size.height);
                    game_state.handle_resize(size.width, size.height);
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {
                self.input_system.handle_window_event(
                    &event,
                    &mut self.input_state,
                    &mut self.window_manager,
                );
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        self.input_system
            .handle_device_event(&event, &mut self.input_state);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.window_manager.exit();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(game_state) = &mut self.game_state {
            self.input_system.update(
                &mut game_state.world,
                &self.input_state,
                &mut game_state.camera,
            );

            self.render_system.render(
                &game_state.world,
                &game_state.camera,
                &game_state.renderer,
                &game_state.shader_program,
            );

            self.window_manager.swap_buffers();
        }

        self.input_state.reset_frame_state();
    }
}
