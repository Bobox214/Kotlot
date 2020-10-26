use ncollide2d::shape::Cuboid;

use super::*;
pub struct Laser {
    pub despawn_timer: Timer,
}

pub struct FireWeaponEvent {
    pub ship_entity: Entity,
}

#[derive(Default)]
pub struct FireWeaponSystemState {
    fire_weapon_listeners: EventReader<FireWeaponEvent>,
}
pub fn fire_weapon_system(
    mut commands: Commands,
    mut state: Local<FireWeaponSystemState>,
    fire_weapon_events: Res<Events<FireWeaponEvent>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio: Res<Audio>,
    (mut collide_world, collide_groups): (ResMut<CollisionWorld<f32, Entity>>, Res<CollideGroups>),
    query_transforms: Query<&Transform>,
) {
    for fire_weapon_event in state.fire_weapon_listeners.iter(&fire_weapon_events) {
        if let Ok(transform) = query_transforms.get::<Transform>(fire_weapon_event.ship_entity) {
            commands
                .spawn_with_ghosts(SpriteComponents {
                    transform: Transform {
                        translation: transform.translation,
                        rotation: transform.rotation,
                        scale: Vec3::splat(0.6),
                    },
                    material: materials.add(asset_server.load("laserRed07.png").into()),
                    ..Default::default()
                })
                .with(Laser {
                    despawn_timer: Timer::from_seconds(2.0, false),
                })
                .with(Movement {
                    speed: (transform.rotation * Vec3::unit_x()).truncate() * 500.0,
                    dampening: 1.0,
                });
            let entity = commands.current_entity().unwrap();
            let shape =
                ShapeHandle::new(Cuboid::new(Vector2::new(37.0 * 0.6 * 0.5, 9.0 * 0.6 * 0.5)));
            let (collision_object_handle, _) = collide_world.add(
                Isometry2::new(Vector2::new(0.0, 0.0), na::zero()),
                shape,
                collide_groups.missiles,
                GeometricQueryType::Contacts(0.0, 0.0),
                entity,
            );
            commands.insert(entity, (collision_object_handle,));
            let sound = asset_server.load("sfx_laser1.mp3");
            audio.play(sound);
        }
    }
}

pub fn despawn_laser_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, Mut<Laser>)>,
) {
    for (entity, mut laser) in &mut query.iter() {
        laser.despawn_timer.tick(time.delta_seconds);
        if laser.despawn_timer.finished {
            commands.despawn_with_ghosts(entity);
        }
    }
}
pub fn weapon_system(time: Res<Time>, mut query: Query<Mut<Weapon>>) {
    for mut weapon in &mut query.iter() {
        weapon.fire_timer.tick(time.delta_seconds);
    }
}
