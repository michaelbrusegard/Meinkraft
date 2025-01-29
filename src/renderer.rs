use crate::gl;

pub struct Renderer {
    gl: gl::Gl,
}

impl Renderer {
    pub fn new<D: glutin::display::GlDisplay>(gl_display: &D) -> Self {
        let gl = unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            gl.Enable(gl::DEPTH_TEST);
            gl
        };

        Self { gl }
    }

    pub fn draw(&self) {
        unsafe {
            self.gl.ClearColor(0.1, 0.1, 0.1, 0.9);
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}
