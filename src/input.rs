use super::*;
use bevy::app::AppExit;
use bevy_prototype_input_map::{InputMap, OnActionActive};

const ACTION_MOVE: &str = "MOVE";
const ACTION_SHOOT_1: &str = "SHOOT_1";
const ACTION_QUIT_APP: &str = "QUIT_APP";
const ACTION_RCS_L: &str = "RCS_LEFT";
const ACTION_RCS_R: &str = "RCS_RIGHT";

pub fn setup_input(mut input_map: ResMut<InputMap>) {
    input_map
        .bind_mouse_button_pressed(MouseButton::Left, ACTION_MOVE)
        .bind_keyboard_pressed(KeyCode::Space, ACTION_SHOOT_1)
        .bind_keyboard_pressed(KeyCode::A, ACTION_RCS_L)
        .bind_keyboard_pressed(KeyCode::D, ACTION_RCS_R)
        .bind_keyboard_pressed(KeyCode::F4, ACTION_QUIT_APP);
}
#[derive(Default)]
pub struct ActionSystemState {
    active_reader: EventReader<OnActionActive>,
}

/// Update User ship orientation based on mouse position.
pub fn orientation_system(
    cursor_world_pos: Res<Cursor2dWorldPos>,
    mut query_spaceship: Query<With<UserControlled, Mut<Transform>>>,
) {
    for mut ship_transform in &mut query_spaceship.iter() {
        let world_x = cursor_world_pos.world_pos.x();
        let world_y = cursor_world_pos.world_pos.y();
        let ship_x = ship_transform.translation.x();
        let ship_y = ship_transform.translation.y();
        ship_transform.rotation = Quat::from_rotation_z((world_y - ship_y).atan2(world_x - ship_x));
    }
}

pub fn action_system(
    mut state: Local<ActionSystemState>,
    time: Res<Time>,
    cursor_world_pos: Res<Cursor2dWorldPos>,
    action_active_events: Res<Events<OnActionActive>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut query_spaceship: Query<With<UserControlled, (&Spaceship, Mut<Velocity>, Mut<Transform>)>>,
) {
    for active_event in state.active_reader.iter(&action_active_events) {
        if active_event.action == ACTION_QUIT_APP {
            app_exit_events.send(AppExit);
        } else {
            for (ship, mut velocity, ship_transform) in &mut query_spaceship.iter() {
                let world_x = cursor_world_pos.world_pos.x();
                let world_y = cursor_world_pos.world_pos.y();
                if active_event.action == ACTION_MOVE {
                    velocity.0 =
                        ship.velocity_to(&*ship_transform, world_x, world_y, time.delta_seconds);
                }
                if active_event.action == ACTION_SHOOT_1 {
                    println!("SHOOT");
                }
            }
        }
    }
}
