use crate::gl;
use crate::resources::{Camera, GlState, MeshRegistry};
use crate::shaders::ShaderProgram;
use crate::systems;
use glam::Vec3;
use hecs::World;
use std::ffi::CString;

pub struct Renderer {
    gl_state: GlState,
    shader_program: ShaderProgram,
    world: World,
    camera: Camera,
    mesh_registry: MeshRegistry,
}

impl Renderer {
    pub fn new<D: glutin::display::GlDisplay>(gl_display: &D) -> Self {
        let gl = unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            gl.Enable(gl::DEPTH_TEST);
            gl
        };

        let gl_state = GlState::new(gl.clone());
        let shader_program = ShaderProgram::new(&gl);
        let mut mesh_registry = MeshRegistry::new();

        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 6.0), // Camera position
            Vec3::new(0.0, 0.0, 0.0), // Look at point
            Vec3::new(0.0, 1.0, 0.0), // Up vector
            800.0 / 600.0,            // Aspect ratio
        );

        let mut world = World::new();

        let cube_mesh_id = crate::ecs::setup_world(&mut world, &mut mesh_registry);

        let mut renderer = Self {
            gl_state,
            shader_program,
            world,
            camera,
            mesh_registry,
        };

        renderer.setup_mesh_buffers();

        renderer
    }

    fn setup_mesh_buffers(&mut self) {
        for (mesh_id, mesh) in self.mesh_registry.meshes.iter().enumerate() {
            unsafe {
                let mut vao = 0;
                self.gl_state.gl.GenVertexArrays(1, &mut vao);
                self.gl_state.gl.BindVertexArray(vao);

                let mut vbo = 0;
                self.gl_state.gl.GenBuffers(1, &mut vbo);
                self.gl_state.gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
                self.gl_state.gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (mesh.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                    mesh.vertices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                let mut ebo = 0;
                self.gl_state.gl.GenBuffers(1, &mut ebo);
                self.gl_state.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
                self.gl_state.gl.BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (mesh.indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                    mesh.indices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                self.gl_state.gl.VertexAttribPointer(
                    0,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    3 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                    std::ptr::null(),
                );
                self.gl_state.gl.EnableVertexAttribArray(0);

                self.gl_state.vaos.insert(mesh_id, vao);
                self.gl_state.ebos.insert(mesh_id, ebo);
            }
        }
    }

    pub fn draw(&self) {
        systems::render_system(
            &self.world,
            &self.gl_state,
            &self.camera,
            &self.shader_program,
        );
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            self.gl_state.gl.Viewport(0, 0, width, height);
        }

        self.camera.update_aspect_ratio(width as f32, height as f32);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        for (_, vao) in &self.gl_state.vaos {
            unsafe {
                self.gl_state.gl.DeleteVertexArrays(1, vao);
            }
        }

        for (_, ebo) in &self.gl_state.ebos {
            unsafe {
                self.gl_state.gl.DeleteBuffers(1, ebo);
            }
        }
    }
}
