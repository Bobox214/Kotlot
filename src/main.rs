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
use std::f32::consts::PI;

const CAMERA_SCALE: f32 = 1.0;
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;

mod arena;
mod collision;
mod input;
mod weapons;
use arena::*;
use collision::*;
use input::*;
use weapons::*;

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
        .add_plugin(InputMapPlugin::default())
        .add_event::<FireWeaponEvent>()
        .add_startup_system_to_stage(startup_stage::PRE_STARTUP, setup_ncollide.system())
        .add_startup_system(setup_input.system())
        .add_startup_system(setup.system())
        .add_startup_system(spawn_background.system())
        .add_startup_system_to_stage(startup_stage::POST_STARTUP, spawn_arena_markers.system())
        .add_system(action_system.system())
        .add_system(fire_weapon_system.system())
        .add_system(position_system.system())
        .add_system(camera_follow_system.system())
        .add_system(orientation_system.system())
        .add_system(collide_position_system.system())
        .add_system(collision_system.system())
        .add_system(spriteghost_quadrant_system.system()) // After camera_follow to catch Arena.shown mutations
        .add_system(spriteghost_sync_system.system())
        .add_system(despawn_laser_system.system())
        .add_system(weapon_system.system())
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
pub struct Spaceship {
    pub max_angvel: f32,
    pub max_linvel: f32,
    pub max_latvel: f32,
}
pub struct Weapon {
    pub fire_timer: Timer,
}
impl Spaceship {
    /// Compute the speed to reach world coordinate, within ship limits.
    pub fn velocity_to(
        &self,
        ship_transform: &Transform,
        world_x: f32,
        world_y: f32,
        delta_seconds: f32,
    ) -> Vec2 {
        let (ship_vec, mut ship_angle) = ship_transform.rotation.to_axis_angle();
        // ship_vec can be Z or -Z;
        let delta_x = world_x - ship_transform.translation.x();
        let delta_y = world_y - ship_transform.translation.y();
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
    mut collide_world: ResMut<CollisionWorld<f32, Entity>>,
    collide_groups: Res<CollideGroups>,
) {
    let camera_entity = commands
        .spawn(Camera2dComponents {
            transform: Transform::from_scale(Vec3::new(CAMERA_SCALE, CAMERA_SCALE, CAMERA_SCALE)),
            ..Default::default()
        })
        .current_entity()
        .unwrap();
    commands.spawn(UiCameraComponents::default());
    commands
        .spawn(SpriteComponents {
            material: materials.add(asset_server.load("playerShip1_red.png").into()),
            //sprite: Sprite::new(Vec2::new(33.0 * 2.0, 33.0 * 2.0)),
            //material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_scale(Vec3::splat(0.3)),
            ..Default::default()
        })
        .with(UserControlled {})
        .with(Movement {
            speed: Vec2::zero(),
            dampening: 0.1,
        })
        .with(Spaceship {
            max_angvel: 2.0 * PI,
            max_linvel: 300.0,
            max_latvel: 300.0,
        })
        .with(FollowedCamera(camera_entity))
        .with(Weapon {
            fire_timer: Timer::from_seconds(0.2, false),
        });
    let shape = ShapeHandle::new(Ball::new(99.0 * 0.3 * 0.5));
    let entity = commands.current_entity().unwrap();
    let (collision_object_handle, _) = collide_world.add(
        Isometry2::new(Vector2::new(0.0, 0.0), na::zero()),
        shape,
        collide_groups.ships,
        GeometricQueryType::Contacts(0.0, 0.0),
        entity,
    );
    commands.insert(entity, (collision_object_handle,));
    commands.insert_resource(Arena {
        size: Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
        shown: ArenaQuadrant::NW,
    });
}
pub fn spawn_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut collide_world: ResMut<CollisionWorld<f32, Entity>>,
    collide_groups: Res<CollideGroups>,
) {
    commands.spawn_with_ghosts(SpriteComponents {
        //material: materials.add(
        //    asset_server
        //        .load("pexels-francesco-ungaro-998641.png")
        //        .into(),
        //),
        sprite: Sprite::new(Vec2::new(1280.0, 853.0)),
        material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, -10.0),
            scale: Vec3::new(CAMERA_SCALE, CAMERA_SCALE, CAMERA_SCALE),
            ..Default::default()
        },
        ..Default::default()
    });
    commands.spawn_with_ghosts(SpriteComponents {
        material: materials.add(asset_server.load("spaceMeteors_001.png").into()),
        //sprite: Sprite::new(Vec2::new(215.0, 211.0)),
        //material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
        transform: Transform {
            translation: Vec3::new(400.0, -200.0, -8.0),
            scale: Vec3::splat(0.5),
            ..Default::default()
        },
        ..Default::default()
    });
    let entity = commands.current_entity().unwrap();
    let shape = ShapeHandle::new(Ball::new(215.0 * 0.5 * 0.5));
    let (collision_object_handle, _) = collide_world.add(
        Isometry2::new(Vector2::new(0.0, 0.0), na::zero()),
        shape,
        collide_groups.asteroids,
        GeometricQueryType::Contacts(0.0, 0.0),
        entity,
    );
    commands.insert(entity, (collision_object_handle,));
}
fn camera_follow_system(
    mut arena: ResMut<Arena>,
    mut query_transform: Query<(&FollowedCamera, Changed<Transform>)>,
    query_camera: Query<With<Camera, Mut<Transform>>>,
) {
    for (followed_camera, transform) in &mut query_transform.iter() {
        if let Ok(mut camera_transform) = query_camera.get_mut::<Transform>(followed_camera.0) {
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
