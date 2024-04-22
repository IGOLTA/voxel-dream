use bevy::{input::mouse::MouseMotion, prelude::*};
use std::f32::consts::PI;
use crate::controls::Controls;

pub struct Player;

impl Plugin for Player {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                free_view_translation,
                free_view_rotation
            ))
            .add_systems(Startup, setup);
        
    }
}

#[derive(Component)]
pub struct FreeViewMovment {
    pub move_speed: f32,
    pub view_sensvity: f32,
    pub fast_move_speed: f32,
}

fn free_view_rotation(
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &FreeViewMovment)>
) {
    let (mut transform, free_view_movment) = query.single_mut();
    
    for mouse_motion in motion_evr.read() {
        let mut eulers = transform.rotation.to_euler(EulerRot::YXZ);

        eulers.0 += -mouse_motion.delta.x * free_view_movment.view_sensvity;
        eulers.1 += -mouse_motion.delta.y * free_view_movment.view_sensvity;

        eulers.1 = eulers.1.clamp(-PI/2., PI/2.);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, eulers.0, eulers.1, eulers.2);
    }
}

fn free_view_translation(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    controls: Res<Controls>,
    mut query: Query<(&mut Transform, &FreeViewMovment)>
) {
    let (mut transform, free_view_movment) = query.single_mut();
    
    let mut move_direction: Vec3 = Vec3::ZERO;

    if keys.pressed(controls.move_forward) {
        move_direction += Vec3::from(transform.forward());
    }
    if keys.pressed(controls.move_backward) {
        move_direction += Vec3::from(transform.back());
    }
    if keys.pressed(controls.move_left) {
        move_direction += Vec3::from(transform.left())
    }
    if keys.pressed(controls.move_right) {
        move_direction += Vec3::from(transform.right())
    }
    if keys.pressed(controls.move_upward) {
        move_direction += Vec3::from(transform.up())
    }
    if keys.pressed(controls.move_downward) {
        move_direction += Vec3::from(transform.down())
    }

    move_direction = move_direction.clamp_length_max(1.);

    transform.translation += 
        move_direction * 
        if keys.pressed(controls.move_faster) { free_view_movment.fast_move_speed }
        else { free_view_movment.move_speed } *
        time.delta().as_secs_f32();
}

fn setup (
    mut commands: Commands
) {
    commands.spawn((
        FreeViewMovment {
            move_speed: 5.,
            fast_move_speed: 20., 
            view_sensvity: 0.0005,
        },
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default() 
        }
    ));
}