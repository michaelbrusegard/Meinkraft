use crate::components::{
    chunk_coord_to_aabb_center, ChunkCoord, Renderable, Transform, CHUNK_EXTENTS,
};
use crate::state::GameState;
use glam::Vec3;

pub struct RenderSystem {}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem {}
    }

    pub fn render(&self, game_state: &mut GameState) {
        let time = game_state.time_of_day;
        let sky_color = calculate_sky_color(time);
        let light_level = calculate_light_level(time);

        game_state.renderer.clear(sky_color);
        game_state.shader_program.use_program();

        let view_matrix = game_state.camera.view_matrix();
        let projection_matrix = game_state.camera.projection_matrix();
        let frustum = game_state.camera.frustum();

        game_state
            .shader_program
            .set_uniform_mat4("viewMatrix", &view_matrix);
        game_state
            .shader_program
            .set_uniform_mat4("projectionMatrix", &projection_matrix);

        game_state
            .texture_manager
            .bind_texture_array(crate::gl::TEXTURE0);
        game_state.shader_program.set_uniform_int("blockTexture", 0);
        game_state
            .shader_program
            .set_uniform_float("lightLevel", light_level);

        for (_entity, (transform, renderable, chunk_coord)) in game_state
            .world
            .query::<(&Transform, &Renderable, &ChunkCoord)>()
            .iter()
        {
            let aabb_center = chunk_coord_to_aabb_center(*chunk_coord);
            if !frustum.intersects_aabb(aabb_center, CHUNK_EXTENTS) {
                continue;
            }

            if let Some(opaque_mesh_id) = renderable.opaque_mesh_id {
                if let Some(mesh) = game_state.mesh_registry.meshes.get(&opaque_mesh_id) {
                    if let Some(vao) = game_state.renderer.vaos.get(&opaque_mesh_id) {
                        let model_matrix = transform.model_matrix();
                        game_state
                            .shader_program
                            .set_uniform_mat4("modelMatrix", &model_matrix);

                        unsafe {
                            game_state.renderer.gl.BindVertexArray(*vao);
                            let index_count = mesh.indices.len() as i32;
                            if index_count > 0 {
                                game_state.renderer.gl.DrawElements(
                                    crate::gl::TRIANGLES,
                                    index_count,
                                    crate::gl::UNSIGNED_INT,
                                    std::ptr::null(),
                                );
                            }
                        }
                    }
                }
            }
        }

        for (_entity, (transform, renderable, chunk_coord)) in game_state
            .world
            .query::<(&Transform, &Renderable, &ChunkCoord)>()
            .iter()
        {
            let aabb_center = chunk_coord_to_aabb_center(*chunk_coord);
            if !frustum.intersects_aabb(aabb_center, CHUNK_EXTENTS) {
                continue;
            }

            if let Some(transparent_mesh_id) = renderable.transparent_mesh_id {
                if let Some(mesh) = game_state.mesh_registry.meshes.get(&transparent_mesh_id) {
                    if let Some(vao) = game_state.renderer.vaos.get(&transparent_mesh_id) {
                        let model_matrix = transform.model_matrix();
                        game_state
                            .shader_program
                            .set_uniform_mat4("modelMatrix", &model_matrix);
                        unsafe {
                            game_state.renderer.gl.BindVertexArray(*vao);
                            let index_count = mesh.indices.len() as i32;
                            if index_count > 0 {
                                game_state.renderer.gl.DrawElements(
                                    crate::gl::TRIANGLES,
                                    index_count,
                                    crate::gl::UNSIGNED_INT,
                                    std::ptr::null(),
                                );
                            }
                        }
                    }
                }
            }
        }

        unsafe {
            game_state.renderer.gl.BindVertexArray(0);
        }
    }
}

fn calculate_light_level(time: f32) -> f32 {
    let min_light = 0.2;
    let max_light = 1.0;

    let dawn_start = 0.22;
    let day_start = 0.28;
    let dusk_start = 0.72;
    let night_start = 0.78;

    if time >= day_start && time <= dusk_start {
        max_light
    } else if time > dawn_start && time < day_start {
        let factor = (time - dawn_start) / (day_start - dawn_start);
        min_light + (max_light - min_light) * factor
    } else if time > dusk_start && time < night_start {
        let factor = (time - dusk_start) / (night_start - dusk_start);
        max_light - (max_light - min_light) * factor
    } else {
        min_light
    }
}

fn calculate_sky_color(time: f32) -> Vec3 {
    let midnight_color = Vec3::new(0.01, 0.01, 0.05);
    let dawn_color = Vec3::new(0.8, 0.4, 0.2);
    let noon_color = Vec3::new(0.5, 0.8, 1.0);
    let dusk_color = Vec3::new(0.9, 0.5, 0.3);

    let dawn_start = 0.23;
    let day_start = 0.27;
    let dusk_start = 0.73;
    let night_start = 0.77;

    if time < dawn_start {
        let factor = time / dawn_start;
        midnight_color.lerp(dawn_color, factor)
    } else if time < day_start {
        let factor = (time - dawn_start) / (day_start - dawn_start);
        dawn_color.lerp(noon_color, factor)
    } else if time < dusk_start {
        noon_color
    } else if time < night_start {
        let factor = (time - dusk_start) / (night_start - dusk_start);
        noon_color.lerp(dusk_color, factor)
    } else {
        let factor = (time - night_start) / (1.0 - night_start);
        dusk_color.lerp(midnight_color, factor)
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new()
    }
}
