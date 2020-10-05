use bevy::{
    input::mouse::MouseButtonInput, prelude::*, render::camera::OrthographicProjection,
    window::CursorMoved,
};

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
        .add_startup_system(setup.system())
        .add_startup_system(spawn_crosshair.system())
        .add_system(spawn_spaceship.system())
        .add_startup_system(spawn_background.system())
        .add_system(crosshair_system.system())
        .add_system(position_system.system())
        .add_system(camera_follow_system.system())
        .run();
}

pub struct CameraFollow(Option<Entity>);

pub fn setup(mut commands: Commands) {
    commands
        .spawn(Camera2dComponents {
            transform: Transform::from_scale(CAMERA_SCALE),
            ..Default::default()
        })
        .with(CameraFollow(None))
        .spawn(UiCameraComponents::default());
}

struct Velocity(Vec2);
struct Crosshair {}
struct Spaceship {}

pub fn spawn_crosshair(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("assets/crosshair066.png").unwrap().into()),
            transform: Transform::from_scale(0.3),
            ..Default::default()
        })
        .with(Crosshair {});
}

pub fn spawn_spaceship(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<Mut<CameraFollow>>,
) {
    for mut camera_follow in &mut query.iter() {
        if camera_follow.0.is_none() {
            let entity = commands
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
                .current_entity();
            camera_follow.0 = entity;
        }
    }
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
#[derive(Default)]
struct CrosshairLocalState {
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
}

/// This system prints out all mouse events as they come in
fn crosshair_system(
    mut state: Local<CrosshairLocalState>,
    mouse_button_input_events: Res<Events<MouseButtonInput>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    mouse_button: Res<Input<MouseButton>>,
    mut query: Query<(&Crosshair, Mut<Transform>)>,
    mut query_camera: Query<(&CameraFollow, &OrthographicProjection, &Transform)>,
    mut query_spaceship: Query<(&Spaceship, Mut<Velocity>, Mut<Transform>)>,
) {
    for _event in state
        .mouse_button_event_reader
        .iter(&mouse_button_input_events)
    {}

    for event in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
        for (_camera, camera_ortho, camera_transform) in &mut query_camera.iter() {
            let world_x =
                event.position.x() + camera_ortho.left + camera_transform.translation().x();
            let world_y =
                event.position.y() + camera_ortho.bottom + camera_transform.translation().y();
            for (_crosshair, mut transform) in &mut query.iter() {
                *transform.translation_mut().x_mut() = world_x;
                *transform.translation_mut().y_mut() = world_y;
                if mouse_button.pressed(MouseButton::Left) {
                    for (_ship, mut velocity, mut ship_transform) in &mut query_spaceship.iter() {
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
        }
    }
}

fn position_system(time: Res<Time>, mut query: Query<(Mut<Transform>, &Velocity)>) {
    let elapsed = time.delta_seconds;
    for (mut transform, velocity) in &mut query.iter() {
        let translation = transform.translation_mut();
        *translation.x_mut() += velocity.0.x() * elapsed;
        *translation.y_mut() += velocity.0.y() * elapsed;
    }
}

fn camera_follow_system(
    mut query_camera: Query<(&CameraFollow, Mut<Transform>)>,
    query_transform: Query<&Transform>,
) {
    for (camera_follow, mut camera_transform) in &mut query_camera.iter() {
        if let Some(entity) = camera_follow.0 {
            if let Ok(transform) = query_transform.get::<Transform>(entity) {
                camera_transform.set_translation(transform.translation());
            }
        }
    }
}
