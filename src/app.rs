use glutin::config::ConfigTemplateBuilder;
use glutin_winit::DisplayBuilder;
use std::error::Error;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

use crate::resources::GlState;
use crate::state::GameState;
use crate::systems;
use crate::window::WindowManager;

pub struct App {
    window_manager: WindowManager,
    game_state: Option<GameState>,
    pub exit_state: Result<(), Box<dyn Error>>,
}

impl App {
    pub fn new(template: ConfigTemplateBuilder, display_builder: DisplayBuilder) -> Self {
        Self {
            window_manager: WindowManager::new(template, display_builder),
            exit_state: Ok(()),
            game_state: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.window_manager.resume(event_loop) {
            Ok(()) => {
                if self.game_state.is_none() {
                    let gl = self.window_manager.create_gl();
                    let gl_state = GlState::new(gl);
                    let mut game_state = GameState::new(gl_state);
                    game_state.initialize();
                    self.game_state = Some(game_state);
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
                self.window_manager.resize(size.width, size.height);

                if let Some(game_state) = self.game_state.as_mut() {
                    game_state.resize(size.width as i32, size.height as i32);
                }
            }
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            _ => (),
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.window_manager.exit();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(game_state) = self.game_state.as_ref() {
            systems::render_system(game_state);
            self.window_manager.swap_buffers();
        }
    }
}
