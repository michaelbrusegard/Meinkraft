mod gl;
mod renderer;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use glutin_winit::GlWindow;
use raw_window_handle::HasWindowHandle;
use renderer::Renderer;
use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Default)]
struct App {
    window: Option<Window>,
    gl_context: Option<PossiblyCurrentContext>,
    gl_surface: Option<Surface<WindowSurface>>,
    renderer: Option<Box<Renderer>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_depth_size(24)
            .with_stencil_size(8)
            .with_single_buffering(false)
            .with_transparency(true);

        fn window_attributes() -> WindowAttributes {
            Window::default_attributes()
                .with_transparent(true)
                .with_title("Glutin triangle gradient example (press Escape to exit)")
        }

        let display_builder =
            DisplayBuilder::new().with_window_attributes(Some(window_attributes()));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .expect("No valid OpenGL configurations found")
            })
            .expect("Failed to create window and GL config");

        let window = window.expect("Failed to create window");
        let raw_window_handle = window.window_handle().expect("Failed to get window handle");

        let context_attributes = ContextAttributesBuilder::new()
            .with_profile(glutin::context::GlProfile::Core)
            .with_context_api(glutin::context::ContextApi::OpenGl(None))
            .build(Some(raw_window_handle.as_raw()));

        let gl_context = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .expect("Failed to create GL context")
        };

        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");

        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .expect("Failed to create window surface")
        };

        let gl_context = gl_context
            .make_current(&gl_surface)
            .expect("Failed to make context current");

        self.window = Some(window);
        self.gl_context = Some(gl_context);
        self.gl_surface = Some(gl_surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(gl_context), Some(gl_surface), Some(renderer)) = (
                    self.gl_context.as_ref(),
                    self.gl_surface.as_ref(),
                    self.renderer.as_ref(),
                ) {
                    renderer.draw();
                    gl_surface.swap_buffers(gl_context).unwrap();
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                if let (Some(gl_context), Some(gl_surface), Some(renderer)) = (
                    self.gl_context.as_ref(),
                    self.gl_surface.as_ref(),
                    self.renderer.as_ref(),
                ) {
                    gl_surface.resize(
                        gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );
                    renderer.resize(size.width as i32, size.height as i32);
                }
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
