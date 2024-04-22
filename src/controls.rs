use bevy::prelude::*;

#[derive(Resource)]
pub struct Controls {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_upward: KeyCode,
    pub move_downward: KeyCode,
    pub move_faster: KeyCode,

    pub quit_game: KeyCode,

    pub enter_debug_mode: KeyCode,
}

impl Default for Controls {
    fn default() -> Self {
        Self { 
            move_forward: KeyCode::KeyW, 
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyA, 
            move_right: KeyCode::KeyD, 
            move_upward: KeyCode::Space,
            move_downward: KeyCode::ControlLeft,
            move_faster: KeyCode::ShiftLeft,

            quit_game: KeyCode::Escape,

            enter_debug_mode: KeyCode::F3,
        }
    }
}