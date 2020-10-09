use bevy::{prelude::*, render::camera::Camera};
use bevy::{window::WindowId, winit::WinitWindows};
use bevy_contrib_bobox::Cursor2dWorldPos;
use winit::window::CursorIcon;

const CAMERA_SCALE: f32 = 1.0;
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;

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
        .run();
}

pub struct FollowedCamera(Entity);

struct Velocity(Vec2);
struct Spaceship {}

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
        .with(Spaceship {})
        .with(FollowedCamera(camera_entity));
    //let window = windows.get_window(WindowId::primary()).unwrap();
    //window.set_cursor_icon(CursorIcon::Crosshair);
}

pub fn spawn_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(SpriteComponents {
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
    cursor_world_pos: Res<Cursor2dWorldPos>,
    mouse_button: Res<Input<MouseButton>>,
    mut query_spaceship: Query<(&Spaceship, Mut<Velocity>, Mut<Transform>)>,
) {
    if mouse_button.pressed(MouseButton::Left) {
        for (_ship, mut velocity, mut ship_transform) in &mut query_spaceship.iter() {
            let world_x = cursor_world_pos.world_pos.x();
            let world_y = cursor_world_pos.world_pos.y();
            let ship_x = ship_transform.translation().x();
            let ship_y = ship_transform.translation().y();
            let new_velocity = Vec2::new(world_x - ship_x, world_y - ship_y);
            velocity.0 = new_velocity;

            ship_transform.set_rotation(Quat::from_rotation_z(
                libm::atan2f(world_y - ship_y, world_x - ship_x) - 3.1415 / 2.0,
            ))
        }
    }
}

fn position_system(time: Res<Time>, mut query: Query<(Mut<Transform>, Mut<Velocity>)>) {
    let elapsed = time.delta_seconds;
    for (mut transform, mut velocity) in &mut query.iter() {
        let translation = transform.translation_mut();
        *translation.x_mut() += velocity.0.x() * elapsed;
        *translation.y_mut() += velocity.0.y() * elapsed;
        velocity.0 = velocity.0 * 0.1f32.powf(time.delta_seconds);
    }
}

fn camera_follow_system(
    mut query_transform: Query<(&FollowedCamera, Changed<Transform>)>,
    query_camera: Query<With<Camera, Mut<Transform>>>,
) {
    for (followed_camera, transform) in &mut query_transform.iter() {
        if let Ok(mut camera_transform) = query_camera.get_mut::<Transform>(followed_camera.0) {
            camera_transform.set_translation(transform.translation());
        }
    }
}
