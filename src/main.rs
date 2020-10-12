use bevy::{prelude::*, render::camera::Camera};
use bevy::{window::WindowId, winit::WinitWindows};
use bevy_contrib_bobox::Cursor2dWorldPos;
use std::f32::consts::PI;
use winit::window::CursorIcon;

const CAMERA_SCALE: f32 = 1.0;
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;

mod arena;
use arena::*;

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb_u8(5, 5, 10)))
        .add_resource(WindowDescriptor {
            title: "Kotlot".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            ..Default::default()
        })
        .add_default_plugins()
        .add_plugin(bevy_contrib_bobox::Cursor2dWorldPosPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(spawn_background.system())
        .add_system(ship_movement_system.system())
        .add_system(position_system.system())
        .add_system(camera_follow_system.system())
        .add_system(spriteghost_quadrant_system.system()) // After camera_follow to catch Arena.shown mutations
        .add_system(spriteghost_sync_system.system())
        .run();
}

pub struct FollowedCamera(Entity);

pub struct Velocity(pub Vec2);
struct Spaceship {
    max_angvel: f32,
    max_linvel: f32,
}
impl Spaceship {
    /// Compute the velocity to reach world coordinate, within ship limits.
    pub fn velocity_to(
        &self,
        ship_transform: &Transform,
        world_x: f32,
        world_y: f32,
        delta_seconds: f32,
    ) -> Vec2 {
        let (ship_vec, mut ship_angle) = ship_transform.rotation().to_axis_angle();
        // ship_vec can be Z or -Z;
        let delta_x = world_x - ship_transform.translation().x();
        let delta_y = world_y - ship_transform.translation().y();
        ship_angle = ship_angle * ship_vec.z();
        let max_angvel = self.max_angvel * delta_seconds;
        let delta_angle = Vec2::new(ship_angle.cos(), ship_angle.sin())
            .angle_between(Vec2::new(delta_x, delta_y))
            .max(-max_angvel)
            .min(max_angvel);
        let new_angle = ship_angle + delta_angle;
        let velocity = Vec2::new(new_angle.cos(), new_angle.sin()) * self.max_linvel;
        velocity
    }
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<WinitWindows>,
) {
    let camera_entity = commands
        .spawn(Camera2dComponents {
            transform: Transform::from_scale(CAMERA_SCALE),
            ..Default::default()
        })
        .current_entity()
        .unwrap();
    commands.spawn(UiCameraComponents::default());
    commands
        .spawn(SpriteComponents {
            material: materials.add(
                asset_server
                    .load("assets/playerShip1_red.png")
                    .unwrap()
                    .into(),
            ),
            transform: Transform::from_scale(0.3),
            ..Default::default()
        })
        .with(Velocity(Vec2::zero()))
        .with(Spaceship {
            max_angvel: 2.0 * PI,
            max_linvel: 400.0,
        })
        .with(FollowedCamera(camera_entity));
    //let window = windows.get_window(WindowId::primary()).unwrap();
    //window.set_cursor_icon(CursorIcon::Crosshair);
    commands.insert_resource(Arena {
        size: Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        shown: Cardinal::NW,
    });
}
pub fn spawn_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_with_ghosts(SpriteComponents {
        material: materials.add(
            asset_server
                .load("assets/pexels-francesco-ungaro-998641.png")
                .unwrap()
                .into(),
        ),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)).with_scale(CAMERA_SCALE),
        ..Default::default()
    });
}
fn ship_movement_system(
    time: Res<Time>,
    cursor_world_pos: Res<Cursor2dWorldPos>,
    mouse_button: Res<Input<MouseButton>>,
    mut query_spaceship: Query<(&Spaceship, Mut<Velocity>, Mut<Transform>)>,
) {
    if mouse_button.pressed(MouseButton::Left) {
        for (ship, mut velocity, mut ship_transform) in &mut query_spaceship.iter() {
            let world_x = cursor_world_pos.world_pos.x();
            let world_y = cursor_world_pos.world_pos.y();
            velocity.0 = ship.velocity_to(&*ship_transform, world_x, world_y, time.delta_seconds);

            ship_transform.set_rotation(Quat::from_rotation_z(velocity.0.y().atan2(velocity.0.x())))
        }
    }
}

fn camera_follow_system(
    mut arena: ResMut<Arena>,
    mut query_transform: Query<(&FollowedCamera, Changed<Transform>)>,
    query_camera: Query<With<Camera, Mut<Transform>>>,
) {
    for (followed_camera, transform) in &mut query_transform.iter() {
        if let Ok(mut camera_transform) = query_camera.get_mut::<Transform>(followed_camera.0) {
            camera_transform.set_translation(transform.translation());
            let shown = match (transform.translation().x(), transform.translation().y()) {
                (x, y) if x <= 0.0 && y <= 0.0 => Cardinal::SW,
                (x, y) if x <= 0.0 && y > 0.0 => Cardinal::NW,
                (x, y) if x > 0.0 && y <= 0.0 => Cardinal::SE,
                (x, y) if x > 0.0 && y > 0.0 => Cardinal::NE,
                _ => panic!("Conditions should have catch everything"),
            };
            if arena.shown != shown {
                arena.shown = shown;
            }
        }
    }
}
