use crate::components::{
    chunk_coord_to_aabb_center, ChunkCoord, Renderable, Transform, CHUNK_EXTENTS,
};
use crate::state::GameState;
use glam::{Mat3, Mat4, Quat, Vec3};
use std::f32::consts::PI;

pub struct RenderSystem {}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem {}
    }

    pub fn render(&self, game_state: &mut GameState) {
        let time = game_state.time_of_day;
        let sky_color = calculate_sky_color(time);
        let light_level = calculate_light_level(time);
        let night_factor = (1.0 - (light_level - 0.15) / (1.0 - 0.15)).clamp(0.0, 1.0);

        game_state.renderer.clear(sky_color);

        let view_matrix = game_state.camera.view_matrix();
        let projection_matrix = game_state.camera.projection_matrix();
        let frustum = game_state.camera.frustum();
        let camera_pos = game_state.camera.position;
        let camera_z_far = game_state.camera.z_far();

        if night_factor > 0.0 {
            game_state.star_shader_program.use_program();

            unsafe {
                game_state.renderer.gl.Enable(crate::gl::DEPTH_TEST);
                game_state.renderer.gl.DepthMask(crate::gl::TRUE);
                game_state.renderer.gl.Enable(crate::gl::PROGRAM_POINT_SIZE);
            }

            let view_matrix_no_translation = Mat4::from_mat3(Mat3::from_mat4(view_matrix));
            game_state
                .star_shader_program
                .set_uniform_mat4("viewMatrix", &view_matrix_no_translation);
            game_state
                .star_shader_program
                .set_uniform_mat4("projectionMatrix", &projection_matrix);

            let star_distance = camera_z_far * 0.95;
            game_state
                .star_shader_program
                .set_uniform_float("starDistance", star_distance);
            game_state
                .star_shader_program
                .set_uniform_float("time", game_state.total_time);
            game_state
                .star_shader_program
                .set_uniform_float("nightFactor", night_factor);

            game_state.renderer.bind_star_vao();
            unsafe {
                game_state.renderer.gl.DrawArrays(
                    crate::gl::POINTS,
                    0,
                    game_state.renderer.num_stars as i32,
                );
                game_state
                    .renderer
                    .gl
                    .Disable(crate::gl::PROGRAM_POINT_SIZE);
            }
        }

        game_state.shader_program.use_program();

        game_state
            .shader_program
            .set_uniform_mat4("viewMatrix", &view_matrix);
        game_state
            .shader_program
            .set_uniform_mat4("projectionMatrix", &projection_matrix);
        game_state
            .shader_program
            .set_uniform_bool("isCelestial", true);
        game_state
            .texture_manager
            .bind_texture_array(crate::gl::TEXTURE0);
        game_state.shader_program.set_uniform_int("blockTexture", 0);
        game_state
            .shader_program
            .set_uniform_float("lightLevel", light_level);

        let sun_layer = game_state
            .texture_manager
            .get_layer_index("sun")
            .unwrap_or(0.0);
        let moon_layer = game_state
            .texture_manager
            .get_layer_index("moon")
            .unwrap_or(0.0);
        let celestial_distance = camera_z_far * 0.9;
        let celestial_scale = camera_z_far * 0.05;

        let angle = time * 2.0 * PI;
        let sun_dir = Vec3::new(angle.sin(), (angle + PI).cos(), 0.0).normalize();
        let moon_dir = -sun_dir;

        let sun_pos = camera_pos + sun_dir * celestial_distance;
        let moon_pos = camera_pos + moon_dir * celestial_distance;

        unsafe {
            game_state.renderer.gl.Disable(crate::gl::DEPTH_TEST);
            game_state.renderer.gl.DepthMask(crate::gl::FALSE);
        }

        game_state.renderer.bind_celestial_vao();

        let sun_forward = (camera_pos - sun_pos).normalize();
        let sun_rotation = Quat::from_rotation_arc(Vec3::Z, sun_forward);
        let sun_model_matrix = Mat4::from_scale_rotation_translation(
            Vec3::splat(celestial_scale),
            sun_rotation,
            sun_pos,
        );
        game_state
            .shader_program
            .set_uniform_mat4("modelMatrix", &sun_model_matrix);
        game_state
            .shader_program
            .set_uniform_float("celestialLayerIndex", sun_layer);
        unsafe {
            game_state.renderer.gl.DrawElements(
                crate::gl::TRIANGLES,
                6,
                crate::gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        let moon_forward = (camera_pos - moon_pos).normalize();
        let moon_rotation = Quat::from_rotation_arc(Vec3::Z, moon_forward);
        let moon_model_matrix = Mat4::from_scale_rotation_translation(
            Vec3::splat(celestial_scale),
            moon_rotation,
            moon_pos,
        );
        game_state
            .shader_program
            .set_uniform_mat4("modelMatrix", &moon_model_matrix);
        game_state
            .shader_program
            .set_uniform_float("celestialLayerIndex", moon_layer);
        unsafe {
            game_state.renderer.gl.DrawElements(
                crate::gl::TRIANGLES,
                6,
                crate::gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        unsafe {
            game_state.renderer.gl.Enable(crate::gl::DEPTH_TEST);
            game_state.renderer.gl.DepthMask(crate::gl::TRUE);
        }

        game_state
            .shader_program
            .set_uniform_bool("isCelestial", false);
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
    let min_light = 0.15;
    let max_light = 1.0;

    let sunrise_center = 0.22;
    let sunset_center = 0.72;
    let transition_duration = 0.05;
    let sunrise_start = sunrise_center - transition_duration;
    let sunrise_end = sunrise_center + transition_duration;
    let sunset_start = sunset_center - transition_duration;
    let sunset_end = sunset_center + transition_duration;

    if time >= sunrise_end && time < sunset_start {
        max_light
    } else if time >= sunrise_start && time < sunrise_end {
        let factor = (time - sunrise_start) / (sunrise_end - sunrise_start);
        min_light + (max_light - min_light) * factor.clamp(0.0, 1.0)
    } else if time >= sunset_start && time < sunset_end {
        let factor = (time - sunset_start) / (sunset_end - sunset_start);
        max_light - (max_light - min_light) * factor.clamp(0.0, 1.0)
    } else {
        min_light
    }
}

fn calculate_sky_color(time: f32) -> Vec3 {
    let midnight_color = Vec3::new(0.01, 0.01, 0.05);
    let noon_color = Vec3::new(0.5, 0.8, 1.0);
    let sunrise_peak_color = Vec3::new(0.9, 0.6, 0.3);
    let sunset_peak_color = Vec3::new(0.9, 0.5, 0.3);

    let sunrise_center = 0.22;
    let sunset_center = 0.72;
    let transition_duration = 0.05;
    let sunrise_start = sunrise_center - transition_duration;
    let sunrise_end = sunrise_center + transition_duration;
    let sunset_start = sunset_center - transition_duration;
    let sunset_end = sunset_center + transition_duration;

    if time >= sunrise_end && time < sunset_start {
        noon_color
    } else if time >= sunrise_start && time < sunrise_end {
        let factor = ((time - sunrise_start) / (sunrise_end - sunrise_start)).clamp(0.0, 1.0);
        if factor < 0.5 {
            midnight_color.lerp(sunrise_peak_color, factor * 2.0)
        } else {
            sunrise_peak_color.lerp(noon_color, (factor - 0.5) * 2.0)
        }
    } else if time >= sunset_start && time < sunset_end {
        let factor = ((time - sunset_start) / (sunset_end - sunset_start)).clamp(0.0, 1.0);
        if factor < 0.5 {
            noon_color.lerp(sunset_peak_color, factor * 2.0)
        } else {
            sunset_peak_color.lerp(midnight_color, (factor - 0.5) * 2.0)
        }
    } else {
        midnight_color
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new()
    }
}
