mod voxel_world;
mod player;
mod debug;
mod controls;

use bevy::app::AppExit;
use bevy::prelude::*;

use bevy::window::PrimaryWindow;
use debug::Debug;
use controls::Controls;
use voxel_world::VoxelWorld;
use player::Player;

fn main() { 
    App::new()
        .insert_resource(Controls::default())
        .insert_resource(AmbientLight{
            color: Color::WHITE,
            brightness: 1000.,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins((VoxelWorld, Player, Debug))
        .add_systems(Startup, cursor_settings)
        .add_systems(Update, quit_game)
        .run()
}

fn cursor_settings(
    mut query: Query<&mut Window, With<PrimaryWindow>>
) {
    let mut primary_window = query.single_mut();

    //primary_window.cursor.grab_mode = CursorGrabMode::Confined;
    primary_window.cursor.visible = false;
}

fn quit_game (
    keys: Res<ButtonInput<KeyCode>>,
    controls: Res<Controls>,
    mut exit_ev: EventWriter<AppExit>
 ) {
    if keys.just_pressed(controls.quit_game) {
        exit_ev.send(AppExit);
    }
}