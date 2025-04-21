use crate::components::{
    chunk_coord_to_aabb_center, ChunkCoord, Renderable, Transform, CHUNK_EXTENTS,
};
use crate::state::GameState;
use glam::{Mat3, Mat4, Quat, Vec3};
use std::f32::consts::PI;

const MIN_LIGHT_LEVEL: f32 = 0.15;
const MAX_LIGHT_LEVEL: f32 = 1.0;
const SUNRISE_CENTER_TIME: f32 = 0.22;
const SUNSET_CENTER_TIME: f32 = 0.72;
const DAY_NIGHT_TRANSITION_DURATION: f32 = 0.05;

const SUNRISE_START_TIME: f32 = SUNRISE_CENTER_TIME - DAY_NIGHT_TRANSITION_DURATION;
const SUNRISE_END_TIME: f32 = SUNRISE_CENTER_TIME + DAY_NIGHT_TRANSITION_DURATION;
const SUNSET_START_TIME: f32 = SUNSET_CENTER_TIME - DAY_NIGHT_TRANSITION_DURATION;
const SUNSET_END_TIME: f32 = SUNSET_CENTER_TIME + DAY_NIGHT_TRANSITION_DURATION;

const MIDNIGHT_COLOR: Vec3 = Vec3::new(0.01, 0.01, 0.05);
const NOON_COLOR: Vec3 = Vec3::new(0.5, 0.8, 1.0);
const SUNRISE_PEAK_COLOR: Vec3 = Vec3::new(0.9, 0.6, 0.3);
const SUNSET_PEAK_COLOR: Vec3 = Vec3::new(0.9, 0.5, 0.3);

const MIN_AMBIENT_INTENSITY: f32 = 0.25;
const MAX_AMBIENT_INTENSITY: f32 = 1.0;
const MIN_ABSOLUTE_AMBIENT: f32 = 0.15;

pub struct RenderSystem {}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem {}
    }

    pub fn render(&self, game_state: &mut GameState) {
        let time = game_state.time_of_day;
        let sky_color = calculate_sky_color(time);
        let light_level = calculate_light_level(time);
        let night_factor = (1.0
            - (light_level - MIN_LIGHT_LEVEL) / (MAX_LIGHT_LEVEL - MIN_LIGHT_LEVEL))
            .clamp(0.0, 1.0);

        let ambient_intensity =
            MIN_AMBIENT_INTENSITY + (MAX_AMBIENT_INTENSITY - MIN_AMBIENT_INTENSITY) * light_level;
        let ambient_color = sky_color * ambient_intensity;

        let angle = time * 2.0 * PI;
        let sun_dir = Vec3::new(angle.sin(), (angle + PI).cos(), 0.0).normalize();
        let moon_dir = -sun_dir;

        let sun_color = Vec3::new(1.0, 0.98, 0.9);
        let moon_color = Vec3::new(0.15, 0.175, 0.25);

        let sun_blend_factor =
            ((light_level - MIN_LIGHT_LEVEL) / (MAX_LIGHT_LEVEL - MIN_LIGHT_LEVEL)).clamp(0.0, 1.0);

        let light_direction = sun_dir.lerp(moon_dir, 1.0 - sun_blend_factor).normalize();
        let light_color = sun_color.lerp(moon_color, 1.0 - sun_blend_factor);

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
            .set_uniform_vec3("lightDirection", &light_direction);
        game_state
            .shader_program
            .set_uniform_vec3("ambientColor", &ambient_color);
        game_state
            .shader_program
            .set_uniform_vec3("lightColor", &light_color);
        game_state
            .shader_program
            .set_uniform_float("minAmbientContribution", MIN_ABSOLUTE_AMBIENT);
        game_state
            .shader_program
            .set_uniform_vec3("cameraPosition", &camera_pos);
        game_state
            .shader_program
            .set_uniform_float("shininess", 32.0);

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
    if (SUNRISE_END_TIME..SUNSET_START_TIME).contains(&time) {
        MAX_LIGHT_LEVEL
    } else if (SUNRISE_START_TIME..SUNRISE_END_TIME).contains(&time) {
        let factor = (time - SUNRISE_START_TIME) / (SUNRISE_END_TIME - SUNRISE_START_TIME);
        MIN_LIGHT_LEVEL + (MAX_LIGHT_LEVEL - MIN_LIGHT_LEVEL) * factor.clamp(0.0, 1.0)
    } else if (SUNSET_START_TIME..SUNSET_END_TIME).contains(&time) {
        let factor = (time - SUNSET_START_TIME) / (SUNSET_END_TIME - SUNSET_START_TIME);
        MAX_LIGHT_LEVEL - (MAX_LIGHT_LEVEL - MIN_LIGHT_LEVEL) * factor.clamp(0.0, 1.0)
    } else {
        MIN_LIGHT_LEVEL
    }
}

fn calculate_sky_color(time: f32) -> Vec3 {
    if (SUNRISE_END_TIME..SUNSET_START_TIME).contains(&time) {
        NOON_COLOR
    } else if (SUNRISE_START_TIME..SUNRISE_END_TIME).contains(&time) {
        let factor =
            ((time - SUNRISE_START_TIME) / (SUNRISE_END_TIME - SUNRISE_START_TIME)).clamp(0.0, 1.0);
        if factor < 0.5 {
            MIDNIGHT_COLOR.lerp(SUNRISE_PEAK_COLOR, factor * 2.0)
        } else {
            SUNRISE_PEAK_COLOR.lerp(NOON_COLOR, (factor - 0.5) * 2.0)
        }
    } else if (SUNSET_START_TIME..SUNSET_END_TIME).contains(&time) {
        let factor =
            ((time - SUNSET_START_TIME) / (SUNSET_END_TIME - SUNSET_START_TIME)).clamp(0.0, 1.0);
        if factor < 0.5 {
            NOON_COLOR.lerp(SUNSET_PEAK_COLOR, factor * 2.0)
        } else {
            SUNSET_PEAK_COLOR.lerp(MIDNIGHT_COLOR, (factor - 0.5) * 2.0)
        }
    } else {
        MIDNIGHT_COLOR
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new()
    }
}
