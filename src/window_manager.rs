use glutin::config::{Config, ConfigTemplateBuilder, GetGlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, NotCurrentContext, PossiblyCurrentContext, Version,
};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::error::Error;
use std::ffi::CString;
use std::num::NonZeroU32;
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorGrabMode, Window, WindowAttributes};

pub enum GlDisplayCreationState {
    Builder(DisplayBuilder),
    Init,
}

pub struct WindowState {
    pub gl_surface: Surface<WindowSurface>,
    pub window: Window,
}

pub struct WindowManager {
    pub template: ConfigTemplateBuilder,
    pub state: Option<WindowState>,
    pub gl_context: Option<PossiblyCurrentContext>,
    pub gl_display: GlDisplayCreationState,
}

impl WindowManager {
    pub fn new(template: ConfigTemplateBuilder, display_builder: DisplayBuilder) -> Self {
        Self {
            template,
            gl_display: GlDisplayCreationState::Builder(display_builder),
            gl_context: None,
            state: None,
        }
    }

    pub fn resume(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        let (window, gl_config) = match &self.gl_display {
            GlDisplayCreationState::Builder(display_builder) => {
                let (window_option, gl_config) = display_builder.clone().build(
                    event_loop,
                    self.template.clone(),
                    Self::gl_config_picker,
                )?;

                let window = window_option.unwrap();

                self.gl_display = GlDisplayCreationState::Init;
                self.gl_context =
                    Some(Self::create_gl_context(&window, &gl_config).treat_as_possibly_current());

                (window, gl_config)
            }
            GlDisplayCreationState::Init => {
                let gl_config = self.gl_context.as_ref().unwrap().config();
                let window = glutin_winit::finalize_window(
                    event_loop,
                    Self::window_attributes(),
                    &gl_config,
                )?;
                (window, gl_config)
            }
        };

        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");

        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .map_err(|e| format!("Failed to create window surface: {}", e))?
        };

        let gl_context = self
            .gl_context
            .as_ref()
            .ok_or("GL context is not initialized")?;
        gl_context
            .make_current(&gl_surface)
            .map_err(|e| format!("Failed to make GL context current: {}", e))?;

        assert!(self
            .state
            .replace(WindowState { gl_surface, window })
            .is_none());

        Ok(())
    }

    pub fn suspend(&mut self) {
        self.state = None;
        self.gl_context = Some(
            self.gl_context
                .take()
                .unwrap()
                .make_not_current()
                .unwrap()
                .treat_as_possibly_current(),
        );
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(WindowState { gl_surface, .. }) = self.state.as_ref() {
            let gl_context = self.gl_context.as_ref().unwrap();
            gl_surface.resize(
                gl_context,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );
        }
    }

    pub fn exit(&mut self) {
        let _gl_display = self.gl_context.take().unwrap().display();
        self.state = None;
    }

    pub fn swap_buffers(&self) {
        if let Some(WindowState { gl_surface, window }) = self.state.as_ref() {
            let gl_context = self.gl_context.as_ref().unwrap();
            window.request_redraw();
            gl_surface.swap_buffers(gl_context).unwrap();
        }
    }

    pub fn create_gl(&self) -> crate::gl::Gl {
        let gl_config = self.gl_context.as_ref().unwrap().config();

        unsafe {
            let gl = crate::gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_config
                    .display()
                    .get_proc_address(symbol.as_c_str())
                    .cast()
            });

            gl.Enable(crate::gl::DEPTH_TEST);
            gl
        }
    }

    fn create_gl_context(window: &Window, gl_config: &Config) -> NotCurrentContext {
        let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 1))))
            .build(raw_window_handle);

        let gl_display = gl_config.display();

        unsafe {
            gl_display
                .create_context(gl_config, &context_attributes)
                .expect("failed to create OpenGL 4.1 context")
        }
    }

    fn window_attributes() -> WindowAttributes {
        Window::default_attributes().with_title("Meinkraft")
    }

    fn gl_config_picker<'a>(configs: Box<dyn Iterator<Item = Config> + 'a>) -> Config {
        configs
            .reduce(|accum, config| {
                if config.num_samples() > accum.num_samples() {
                    config
                } else {
                    accum
                }
            })
            .unwrap()
    }

    pub fn initialize_window(&mut self) {
        if let Some(window) = self.state.as_ref().map(|s| &s.window) {
            let _ = window.set_cursor_grab(CursorGrabMode::Locked);
            window.set_cursor_visible(false);
        }
    }

    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        self.state.as_ref().map(|s| {
            let size = s.window.inner_size();
            (size.width, size.height)
        })
    }

    pub fn set_cursor_grabbed(&mut self, grab: bool) {
        if let Some(state) = &self.state {
            if grab {
                let _ = state.window.set_cursor_grab(CursorGrabMode::Locked);
                state.window.set_cursor_visible(false);
            } else {
                let _ = state.window.set_cursor_grab(CursorGrabMode::None);
                state.window.set_cursor_visible(true);
            }
        }
    }
}
