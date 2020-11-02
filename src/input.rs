use std::f32::consts::FRAC_PI_2;

use super::*;
use bevy::app::AppExit;
use bevy_prototype_input_map::{InputMap, OnActionActive};
use ncollide2d::pipeline::CollisionObjectSlabHandle;

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

pub struct CursorSelection {
    pub cursor_collider_handle: CollisionObjectSlabHandle,
    pub enemies: Vec<Entity>,
    pub loots: Vec<Entity>,
}
pub fn spawn_cursor_collider(
    mut commands: Commands,
    (mut collide_world, collide_groups): (ResMut<CollisionWorld<f32, Entity>>, Res<CollideGroups>),
) {
    commands.spawn((ColliderType::Cursor,));
    let entity = commands.current_entity().unwrap();
    let shape = ShapeHandle::new(Ball::new(1.0));
    let (cursor_collider_handle, _) = collide_world.add(
        Isometry2::new(Vector2::new(0.0, 0.0), na::zero()),
        shape,
        collide_groups.cursors,
        GeometricQueryType::Contacts(0.0, 0.0),
        entity,
    );
    commands.insert(entity, (cursor_collider_handle,));
    commands.insert_resource(CursorSelection {
        cursor_collider_handle,
        enemies: vec![],
        loots: vec![],
    });
}
pub fn cursor_collider_system(
    cursor_world_pos: Res<Cursor2dWorldPos>,
    cursor_selection: Res<CursorSelection>,
    mut collision_world: ResMut<CollisionWorld<f32, Entity>>,
) {
    let c1 = collision_world
        .get_mut(cursor_selection.cursor_collider_handle)
        .expect("Cursor collision handle no more in the collision world.");
    c1.set_position(Isometry2::new(
        Vector2::new(
            cursor_world_pos.world_pos.x(),
            cursor_world_pos.world_pos.y(),
        ),
        na::zero(),
    ));
}
/// Update User ship orientation based on mouse position.
pub fn orientation_system(
    cursor_world_pos: Res<Cursor2dWorldPos>,
    mut query_spaceship: Query<With<UserControlled, Mut<Transform>>>,
) {
    for mut ship_transform in query_spaceship.iter_mut() {
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
                query_spaceship.iter_mut()
            {
                if active_event.action == ACTION_SHOOT_1 {
                    if weapon.fire_timer.finished {
                        fire_weapon_events.send(FireWeaponEvent {
                            ship_entity: ship_entity,
                            munition_lifespan: weapon.munition_lifespan,
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
