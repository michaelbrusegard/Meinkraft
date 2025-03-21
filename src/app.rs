use crate::input::InputManager;
use crate::scheduler::SystemScheduler;
use crate::state::GameState;
use crate::window::WindowManager;
use glutin::config::ConfigTemplateBuilder;
use std::error::Error;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

pub struct App {
    window_manager: WindowManager,
    input_manager: InputManager,
    game_state: Option<GameState>,
    system_scheduler: SystemScheduler,
    pub exit_state: Result<(), Box<dyn Error>>,
}

impl App {
    pub fn new(template: ConfigTemplateBuilder) -> Self {
        Self {
            window_manager: WindowManager::new(template),
            game_state: None,
            system_scheduler: SystemScheduler::new(),
            input_manager: InputManager::new(),
            exit_state: Ok(()),
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
                if let Some(game_state) = &mut self.game_state {
                    self.input_manager.handle_window_event(
                        &event,
                        &mut game_state.input_state,
                        &mut self.window_manager,
                    );
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(game_state) = &mut self.game_state {
            self.input_manager
                .handle_device_event(&event, &mut game_state.input_state);
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.window_manager.exit();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(game_state) = &mut self.game_state {
            self.system_scheduler.update(game_state);
            self.system_scheduler.render(game_state);

            game_state.input_state.reset_frame_state();
            self.window_manager.swap_buffers();
        }
    }
}
