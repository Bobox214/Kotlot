use bevy::{app::startup_stage, prelude::*, render::camera::Camera};
use bevy_contrib_bobox::Cursor2dWorldPos;
use bevy_prototype_input_map::InputMapPlugin;
use ncollide2d::{
    na,
    na::{Isometry2, Vector2},
    pipeline::GeometricQueryType,
    shape::{Ball, ShapeHandle},
    world::CollisionWorld,
};
use rand::prelude::*;
use std::f32::consts::PI;

const CAMERA_SCALE: f32 = 1.0;
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;

mod arena;
mod armor;
mod collision;
mod input;
mod loot;
mod spaceship;
mod weapon;
use arena::*;
use armor::*;
use collision::*;
use input::*;
use loot::*;
use spaceship::*;
use weapon::*;

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb_u8(5, 5, 10)))
        .add_resource(WindowDescriptor {
            title: "Kotlot".to_string(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            ..Default::default()
        })
        .add_event::<XpEvent>()
        .add_event::<LootEvent>()
        .add_event::<CollisionEvent>()
        .add_default_plugins()
        .add_plugin(bevy_contrib_bobox::Cursor2dWorldPosPlugin)
        .add_plugin(InputMapPlugin::default())
        .add_event::<FireWeaponEvent>()
        .add_startup_system_to_stage(startup_stage::PRE_STARTUP, setup_ncollide.system())
        .add_startup_system(setup_input.system())
        .add_startup_system(setup.system())
        .add_startup_system(spawn_background.system())
        .add_startup_system(spawn_cursor_collider.system())
        .add_startup_system_to_stage(startup_stage::POST_STARTUP, spawn_player_spaceship.system())
        .add_startup_system_to_stage(startup_stage::POST_STARTUP, spawn_arena_markers.system())
        .add_system(spawn_asteroid.system())
        .add_system(action_system.system())
        .add_system(fire_weapon_system.system())
        .add_system(position_system.system())
        .add_system(camera_follow_system.system())
        .add_system(orientation_system.system())
        .add_system(collide_position_system.system())
        .add_system(collision_system.system())
        .add_system(collision_event_system.system())
        .add_system(spriteghost_quadrant_system.system()) // After camera_follow to catch Arena.shown mutations
        .add_system(spriteghost_sync_system.system())
        .add_system(lifespan_system.system())
        .add_system(weapon_system.system())
        .add_system(xp_system.system())
        .add_system(loot_spawn_system.system())
        .add_system(tweenscale_system.system())
        .add_system(cursor_collider_system.system())
        .run();
}

pub struct FollowedCamera(Entity);

#[derive(Debug)]
pub struct Movement {
    pub speed: Vec2,
    /// Defines speed factor after 1s
    pub dampening: f32,
}
pub struct UserControlled {}
pub struct Enemy {
    pub xp: u32,
}
pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dComponents {
        transform: Transform::from_scale(Vec3::new(CAMERA_SCALE, CAMERA_SCALE, CAMERA_SCALE)),
        ..Default::default()
    });
    commands.spawn(UiCameraComponents::default());
    commands.insert_resource(Arena {
        size: Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        shown: ArenaQuadrant::NW,
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
                .load("pexels-francesco-ungaro-998641.png")
                .into(),
        ),
        //sprite: Sprite::new(Vec2::new(1280.0, 853.0)),
        //material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -10.0),
            scale: Vec3::new(CAMERA_SCALE, CAMERA_SCALE, CAMERA_SCALE),
            ..Default::default()
        },
        ..Default::default()
    });
}
pub fn spawn_asteroid(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    arena: Res<Arena>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut collide_world: ResMut<CollisionWorld<f32, Entity>>,
    collide_groups: Res<CollideGroups>,
    enemies: Query<&Enemy>,
    ship_transforms: Query<With<Spaceship, &Transform>>,
) {
    let n_enemies = enemies.iter().len();
    if n_enemies < 1 {
        // Find a far enough position
        let mut rng = thread_rng();
        let mut x;
        let mut y;
        loop {
            x = rng.gen_range(-arena.size.x() / 2.0, arena.size.x() / 2.0);
            y = rng.gen_range(-arena.size.y() / 2.0, arena.size.y() / 2.0);
            if ship_transforms.iter().all(|transform| {
                ((transform.translation.x() - x).powi(2) + (transform.translation.y() - y).powi(2))
                    > 300.0f32.powi(2)
            }) {
                break;
            }
        }
        commands
            .spawn_with_ghosts(SpriteComponents {
                material: materials.add(asset_server.load("spaceMeteors_001.png").into()),
                transform: Transform {
                    translation: Vec3::new(x, y, -8.0),
                    scale: Vec3::splat(0.5),
                    ..Default::default()
                },
                ..Default::default()
            })
            .with(Armor::new(3))
            .with(Enemy { xp: 2 })
            .with(ColliderType::Enemy);
        let entity = commands.current_entity().unwrap();
        let shape = ShapeHandle::new(Ball::new(215.0 * 0.5 * 0.5));
        let (collision_object_handle, _) = collide_world.add(
            Isometry2::new(Vector2::new(x, y), na::zero()),
            shape,
            collide_groups.enemies,
            GeometricQueryType::Contacts(0.0, 0.0),
            entity,
        );
        commands.insert(entity, (collision_object_handle,));
    }
}
fn camera_follow_system(
    mut arena: ResMut<Arena>,
    query_transform: Query<(&FollowedCamera, Changed<Transform>)>,
    mut query_camera: Query<With<Camera, Mut<Transform>>>,
) {
    for (followed_camera, transform) in &mut query_transform.iter() {
        if let Ok(mut camera_transform) =
            query_camera.get_component_mut::<Transform>(followed_camera.0)
        {
            camera_transform.translation = transform.translation;
            let shown = match (transform.translation.x(), transform.translation.y()) {
                (x, y) if x <= 0.0 && y <= 0.0 => ArenaQuadrant::SW,
                (x, y) if x <= 0.0 && y > 0.0 => ArenaQuadrant::NW,
                (x, y) if x > 0.0 && y <= 0.0 => ArenaQuadrant::SE,
                (x, y) if x > 0.0 && y > 0.0 => ArenaQuadrant::NE,
                _ => panic!(
                    "Conditions should have catch everything {} {}",
                    transform.translation.x(),
                    transform.translation.y()
                ),
            };
            if arena.shown != shown {
                arena.shown = shown;
            }
        }
    }
}
