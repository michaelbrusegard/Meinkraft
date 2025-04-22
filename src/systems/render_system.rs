use crate::components::{
    chunk_coord_to_aabb_center, get_chunk_extents, ChunkCoord, Renderable, Transform,
};
use crate::gl;
use crate::resources::Config;
use crate::state::GameState;
use glam::{Mat3, Mat4, Quat, Vec3};
use std::f32::consts::PI;

pub struct RenderSystem {}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem {}
    }

    pub fn render(&self, game_state: &mut GameState) {
        let config = &game_state.config;
        let time = game_state.time_of_day;
        let sky_color = calculate_sky_color(time, config);
        let light_level = calculate_light_level(time, config);
        let night_factor = (1.0
            - (light_level - config.min_light_level)
                / (config.max_light_level - config.min_light_level))
            .clamp(0.0, 1.0);

        let ambient_intensity = config.min_ambient_intensity
            + (config.max_ambient_intensity - config.min_ambient_intensity) * light_level;
        let ambient_color = sky_color * ambient_intensity;

        let angle = time * 2.0 * PI;
        let sun_dir = Vec3::new(angle.sin(), (angle + PI).cos(), 0.0).normalize();
        let moon_dir = -sun_dir;

        let sun_color = Vec3::new(1.0, 0.98, 0.9);
        let moon_color = Vec3::new(0.15, 0.175, 0.25);

        let sun_blend_factor = ((light_level - config.min_light_level)
            / (config.max_light_level - config.min_light_level))
            .clamp(0.0, 1.0);

        let light_direction = sun_dir.lerp(moon_dir, 1.0 - sun_blend_factor).normalize();
        let light_color = sun_color.lerp(moon_color, 1.0 - sun_blend_factor);

        let shadow_distance_world = config.shadow_distance as f32 * config.chunk_width as f32;
        let light_offset_distance = shadow_distance_world * 1.5;
        let light_pos = game_state.camera.position - light_direction * light_offset_distance;

        let light_view_matrix = Mat4::look_at_rh(light_pos, game_state.camera.position, Vec3::Y);

        let shadow_map_world_size = shadow_distance_world * 2.0;
        let world_units_per_texel = shadow_map_world_size / config.shadow_map_resolution as f32;

        let shadow_area_center_world = game_state.camera.position;

        let shadow_area_center_light_view =
            light_view_matrix.transform_point3(shadow_area_center_world);

        let snapped_center_x = (shadow_area_center_light_view.x / world_units_per_texel).floor()
            * world_units_per_texel;
        let snapped_center_y = (shadow_area_center_light_view.y / world_units_per_texel).floor()
            * world_units_per_texel;

        let half_size = shadow_distance_world;
        let snapped_left = snapped_center_x - half_size;
        let snapped_right = snapped_center_x + half_size;
        let snapped_bottom = snapped_center_y - half_size;
        let snapped_top = snapped_center_y + half_size;

        let near_plane = 1.0;
        let far_plane = light_offset_distance + shadow_distance_world;

        let light_projection_matrix = Mat4::orthographic_rh(
            snapped_left,
            snapped_right,
            snapped_bottom,
            snapped_top,
            far_plane,
            near_plane,
        );

        game_state.light_space_matrix = light_projection_matrix * light_view_matrix;

        let mut viewport = [0i32; 4];
        unsafe {
            game_state
                .renderer
                .gl
                .GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
        }
        let window_width = viewport[2];
        let window_height = viewport[3];

        game_state.renderer.bind_shadow_fbo();

        game_state.shadow_shader_program.use_program();
        game_state
            .shadow_shader_program
            .set_uniform_mat4("lightSpaceMatrix", &game_state.light_space_matrix);

        for (_entity, (transform, renderable, _chunk_coord)) in game_state
            .world
            .query::<(&Transform, &Renderable, &ChunkCoord)>()
            .iter()
        {
            if let Some(opaque_mesh_id) = renderable.opaque_mesh_id {
                if let Some(mesh) = game_state.mesh_registry.meshes.get(&opaque_mesh_id) {
                    if let Some(vao) = game_state.renderer.vaos.get(&opaque_mesh_id) {
                        let model_matrix = transform.model_matrix();
                        game_state
                            .shadow_shader_program
                            .set_uniform_mat4("modelMatrix", &model_matrix);

                        unsafe {
                            game_state.renderer.gl.BindVertexArray(*vao);
                            let index_count = mesh.indices.len() as i32;
                            if index_count > 0 {
                                game_state.renderer.gl.DrawElements(
                                    gl::TRIANGLES,
                                    index_count,
                                    gl::UNSIGNED_INT,
                                    std::ptr::null(),
                                );
                            }
                        }
                    }
                }
            }
        }

        game_state
            .renderer
            .unbind_shadow_fbo(window_width, window_height);

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
            .set_uniform_float("minAmbientContribution", config.min_absolute_ambient);
        game_state
            .shader_program
            .set_uniform_vec3("cameraPosition", &camera_pos);
        game_state
            .shader_program
            .set_uniform_float("shininess", config.material_shininess);
        game_state
            .shader_program
            .set_uniform_mat4("lightSpaceMatrix", &game_state.light_space_matrix);
        game_state.renderer.bind_shadow_map_texture(gl::TEXTURE1);
        game_state.shader_program.set_uniform_int("shadowMap", 1);

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

        game_state.texture_manager.bind_texture_array(gl::TEXTURE0);
        game_state.shader_program.set_uniform_int("blockTexture", 0);

        for (_entity, (transform, renderable, chunk_coord)) in game_state
            .world
            .query::<(&Transform, &Renderable, &ChunkCoord)>()
            .iter()
        {
            let aabb_center = chunk_coord_to_aabb_center(&game_state.config, *chunk_coord);
            let chunk_extents = get_chunk_extents(&game_state.config);
            if !frustum.intersects_aabb(aabb_center, chunk_extents) {
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
            let aabb_center = chunk_coord_to_aabb_center(&game_state.config, *chunk_coord);
            let chunk_extents = get_chunk_extents(&game_state.config);
            if !frustum.intersects_aabb(aabb_center, chunk_extents) {
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

fn calculate_light_level(time: f32, config: &Config) -> f32 {
    let sunrise_start = config.sunrise_center_time - config.day_night_transition_duration;
    let sunrise_end = config.sunrise_center_time + config.day_night_transition_duration;
    let sunset_start = config.sunset_center_time - config.day_night_transition_duration;
    let sunset_end = config.sunset_center_time + config.day_night_transition_duration;

    if (sunrise_end..sunset_start).contains(&time) {
        config.max_light_level
    } else if (sunrise_start..sunrise_end).contains(&time) {
        let factor = (time - sunrise_start) / (sunrise_end - sunrise_start);
        config.min_light_level
            + (config.max_light_level - config.min_light_level) * factor.clamp(0.0, 1.0)
    } else if (sunset_start..sunset_end).contains(&time) {
        let factor = (time - sunset_start) / (sunset_end - sunset_start);
        config.max_light_level
            - (config.max_light_level - config.min_light_level) * factor.clamp(0.0, 1.0)
    } else {
        config.min_light_level
    }
}

fn calculate_sky_color(time: f32, config: &Config) -> Vec3 {
    let sunrise_start = config.sunrise_center_time - config.day_night_transition_duration;
    let sunrise_end = config.sunrise_center_time + config.day_night_transition_duration;
    let sunset_start = config.sunset_center_time - config.day_night_transition_duration;
    let sunset_end = config.sunset_center_time + config.day_night_transition_duration;

    if (sunrise_end..sunset_start).contains(&time) {
        config.noon_color
    } else if (sunrise_start..sunrise_end).contains(&time) {
        let factor = ((time - sunrise_start) / (sunrise_end - sunrise_start)).clamp(0.0, 1.0);
        if factor < 0.5 {
            config
                .midnight_color
                .lerp(config.sunrise_peak_color, factor * 2.0)
        } else {
            config
                .sunrise_peak_color
                .lerp(config.noon_color, (factor - 0.5) * 2.0)
        }
    } else if (sunset_start..sunset_end).contains(&time) {
        let factor = ((time - sunset_start) / (sunset_end - sunset_start)).clamp(0.0, 1.0);
        if factor < 0.5 {
            config
                .noon_color
                .lerp(config.sunset_peak_color, factor * 2.0)
        } else {
            config
                .sunset_peak_color
                .lerp(config.midnight_color, (factor - 0.5) * 2.0)
        }
    } else {
        config.midnight_color
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new()
    }
}
