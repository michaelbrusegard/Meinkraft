use crate::resources::InputState;
use hecs::World;
use winit::keyboard::Key;

pub struct InputSystem {}

impl InputSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(
        &self,
        _world: &mut World,
        input_state: &InputState,
        camera: &mut crate::resources::Camera,
    ) {
        let move_speed = 0.1;
        if input_state
            .pressed_keys
            .contains(&Key::Character("w".into()))
        {
            camera.position.z -= move_speed;
            camera.target.z -= move_speed;
        }
        if input_state
            .pressed_keys
            .contains(&Key::Character("s".into()))
        {
            camera.position.z += move_speed;
            camera.target.z += move_speed;
        }
        if input_state
            .pressed_keys
            .contains(&Key::Character("a".into()))
        {
            camera.position.x -= move_speed;
            camera.target.x -= move_speed;
        }
        if input_state
            .pressed_keys
            .contains(&Key::Character("d".into()))
        {
            camera.position.x += move_speed;
            camera.target.x += move_speed;
        }
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new()
    }
}
