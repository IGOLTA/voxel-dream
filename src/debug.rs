mod world;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use world::DebugGizmos;
use crate::controls::Controls;

pub struct Debug;

#[derive(Resource, Default)]
struct DebugModeData {
    in_debug_mode: bool,
    chunk_gizmos: bool,
}

#[derive(Component)]
struct DebugModeEntity {}


#[derive(Event)]
struct DisableDebugMode {}

#[derive(Event)]
struct EnableDebugMode {}

impl Plugin for Debug {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_event::<EnableDebugMode>()
            .add_event::<DisableDebugMode>()
            .init_gizmo_group::<DebugGizmos>()
            .add_systems(Update, (
                debug_mode_toggle_system,
                enable_debug_mode_system,
                disable_debug_mode_system,
                world::chunk_gizmos_toggle,
                world::draw_octree_borders
            ))
            .insert_resource(DebugModeData::default());
    }
}

fn debug_mode_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_mode_data: ResMut<DebugModeData>,
    controls: Res<Controls>,
    mut enable_debug_ev: EventWriter<EnableDebugMode>,
    mut disable_debug_ev: EventWriter<DisableDebugMode>
) {
    if keys.just_pressed(controls.enter_debug_mode) {
        if debug_mode_data.in_debug_mode {
            disable_debug_ev.send(DisableDebugMode{});
            debug_mode_data.in_debug_mode = false;
        } else {
            enable_debug_ev.send(EnableDebugMode{});
            debug_mode_data.in_debug_mode = true;
        }
    }
}

fn enable_debug_mode_system(
    mut commands: Commands,
    mut events: EventReader<EnableDebugMode>,
) {
    for _ in events.read() {
        commands.spawn((
            DebugModeEntity{},
            TextBundle::from_section(
                "Debug mode
                Press F3 to quit
                Press G to toggle chunk gizmos",
                TextStyle {
                    font_size: 20.,
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            }),
        ));
    }

}

fn disable_debug_mode_system(
    mut commands: Commands,
    mut events: EventReader<DisableDebugMode>,
    query: Query<Entity, With<DebugModeEntity>>
) {
    for _ in events.read() {
        for entity in query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }

}
