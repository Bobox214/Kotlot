use std::f32::consts::FRAC_PI_2;

use super::*;
use bevy::app::AppExit;
use bevy_prototype_input_map::{InputMap, OnActionActive};

const ACTION_FORWARD: &str = "FORWARD";
const ACTION_BACKWARD: &str = "BACKWARD";
const ACTION_SHOOT_1: &str = "SHOOT_1";
const ACTION_QUIT_APP: &str = "QUIT_APP";
const ACTION_RCS_L: &str = "RCS_LEFT";
const ACTION_RCS_R: &str = "RCS_RIGHT";

pub fn setup_input(mut input_map: ResMut<InputMap>) {
    input_map
        .bind_mouse_button_pressed(MouseButton::Left, ACTION_SHOOT_1)
        .bind_keyboard_pressed(KeyCode::Space, ACTION_SHOOT_1)
        .bind_keyboard_pressed(KeyCode::W, ACTION_FORWARD)
        .bind_keyboard_pressed(KeyCode::S, ACTION_BACKWARD)
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
    action_active_events: Res<Events<OnActionActive>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut fire_weapon_events: ResMut<Events<FireWeaponEvent>>,
    mut query_spaceship: Query<
        With<
            UserControlled,
            (
                Entity,
                &Spaceship,
                Mut<Movement>,
                Mut<Transform>,
                Mut<Weapon>,
            ),
        >,
    >,
) {
    for active_event in state.active_reader.iter(&action_active_events) {
        if active_event.action == ACTION_QUIT_APP {
            app_exit_events.send(AppExit);
        } else {
            for (ship_entity, ship, mut movement, ship_transform, mut weapon) in
                &mut query_spaceship.iter()
            {
                if active_event.action == ACTION_SHOOT_1 {
                    if weapon.fire_timer.finished {
                        fire_weapon_events.send(FireWeaponEvent {
                            ship_entity: ship_entity,
                        });
                        weapon.fire_timer.reset();
                    }
                }
                if active_event.action == ACTION_FORWARD {
                    movement.speed += (ship_transform.rotation
                        * (Vec3::unit_x() * ship.max_linvel)
                        * time.delta_seconds)
                        .truncate();
                }
                if active_event.action == ACTION_BACKWARD {
                    movement.speed -= (ship_transform.rotation
                        * (Vec3::unit_x() * ship.max_linvel)
                        * time.delta_seconds)
                        .truncate();
                }
                if active_event.action == ACTION_RCS_L {
                    movement.speed += (ship_transform.rotation
                        * Quat::from_rotation_z(FRAC_PI_2)
                        * Vec3::unit_x()
                        * ship.max_latvel
                        * time.delta_seconds)
                        .truncate();
                }
                if active_event.action == ACTION_RCS_R {
                    movement.speed += (ship_transform.rotation
                        * Quat::from_rotation_z(-FRAC_PI_2)
                        * Vec3::unit_x()
                        * ship.max_latvel
                        * time.delta_seconds)
                        .truncate();
                }
            }
        }
    }
}
