use crate::resources::InputState;
use crate::systems::InputSystem;
use glutin::config::ConfigTemplateBuilder;
use glutin_winit::DisplayBuilder;
use hecs::World;
use std::error::Error;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorGrabMode, WindowId};

use crate::resources::{Camera, Config, MeshRegistry, Renderer, ShaderProgram};
use crate::systems::{InitSystem, RenderSystem};
use crate::window_manager::WindowManager;
use glam::Vec3;

pub struct App {
    window_manager: WindowManager,
    world: Option<World>,
    camera: Option<Camera>,
    mesh_registry: Option<MeshRegistry>,
    renderer: Option<Renderer>,
    shader_program: Option<ShaderProgram>,
    init_system: InitSystem,
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
            world: None,
            camera: None,
            mesh_registry: None,
            renderer: None,
            shader_program: None,
            init_system: InitSystem::new(),
            render_system: RenderSystem::new(),
            input_state: InputState::new(),
            input_system: InputSystem::new(config),
        }
    }

    fn initialize_game(&mut self) {
        let gl = self.window_manager.create_gl();
        let mut renderer = Renderer::new(gl);
        let shader_program = ShaderProgram::new(&renderer.gl);

        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 6.0), // Camera position
            Vec3::new(0.0, 0.0, 0.0), // Look at point
            Vec3::new(0.0, 1.0, 0.0), // Up vector
            800.0 / 600.0,            // Aspect ratio
        );

        let mut world = World::new();
        let mut mesh_registry = MeshRegistry::new();

        self.init_system
            .initialize(&mut world, &mut mesh_registry, &mut renderer);

        self.world = Some(world);
        self.camera = Some(camera);
        self.mesh_registry = Some(mesh_registry);
        self.renderer = Some(renderer);
        self.shader_program = Some(shader_program);

        if let Some(window) = self.window_manager.state.as_ref().map(|s| &s.window) {
            let _ = window.set_cursor_grab(CursorGrabMode::Locked);
            window.set_cursor_visible(false);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.window_manager.resume(event_loop) {
            Ok(()) => {
                if self.world.is_none() {
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
                self.window_manager.resize(size.width, size.height);

                if let (Some(renderer), Some(camera)) =
                    (self.renderer.as_ref(), self.camera.as_mut())
                {
                    renderer.resize(size.width as i32, size.height as i32);
                    camera.update_aspect_ratio(size.width as f32, size.height as f32);
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {
                let window = self.window_manager.state.as_ref().map(|s| &s.window);
                self.input_system
                    .handle_window_event(&event, &mut self.input_state, window);
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
        if let (Some(world), Some(camera)) = (self.world.as_mut(), self.camera.as_mut()) {
            self.input_system.update(world, &self.input_state, camera);
        }

        if let (Some(world), Some(camera), Some(renderer), Some(shader_program)) = (
            self.world.as_ref(),
            self.camera.as_ref(),
            self.renderer.as_ref(),
            self.shader_program.as_ref(),
        ) {
            self.render_system
                .render(world, camera, renderer, shader_program);
            self.window_manager.swap_buffers();
        }

        self.input_state.reset_frame_state();
    }
}
