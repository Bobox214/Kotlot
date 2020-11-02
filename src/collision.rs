use super::*;
use ncollide2d::{
    na::{self, Isometry2, Vector2},
    pipeline::{CollisionGroups, CollisionObjectSlabHandle},
    world::CollisionWorld,
};

pub struct CollideGroups {
    pub ships: CollisionGroups,
    pub enemies: CollisionGroups,
    pub missiles: CollisionGroups,
    pub loots: CollisionGroups,
    pub cursors: CollisionGroups,
}
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum ColliderType {
    Ship,
    Enemy,
    Missile,
    Loot,
    Cursor,
}
pub fn setup_ncollide(mut commands: Commands) {
    let world = CollisionWorld::<f32, Entity>::new(0.02);
    let ships = CollisionGroups::new()
        .with_membership(&[1])
        .with_whitelist(&[1, 2, 3, 4])
        .with_blacklist(&[]);
    let enemies = CollisionGroups::new()
        .with_membership(&[2])
        .with_whitelist(&[1, 2, 3, 5])
        .with_blacklist(&[]);
    let missiles = CollisionGroups::new()
        .with_membership(&[3])
        .with_whitelist(&[2])
        .with_blacklist(&[]);
    let loots = CollisionGroups::new()
        .with_membership(&[4])
        .with_whitelist(&[1, 5])
        .with_blacklist(&[]);
    let cursors = CollisionGroups::new()
        .with_membership(&[5])
        .with_whitelist(&[2, 4])
        .with_blacklist(&[]);
    commands.insert_resource(CollideGroups {
        ships,
        enemies,
        missiles,
        loots,
        cursors,
    });
    commands.insert_resource(world);
}

pub fn collide_position_system(
    mut world: ResMut<CollisionWorld<f32, Entity>>,
    query: Query<(&Transform, &CollisionObjectSlabHandle)>,
) {
    for (transform, &handle) in &mut query.iter() {
        let collision_object = world.get_mut(handle).unwrap();
        collision_object.set_position(Isometry2::new(
            Vector2::new(transform.translation.x(), transform.translation.y()),
            na::zero(),
        ));
    }
}
pub enum CollisionEvent {
    MissileToEnemy(Entity, Entity),
    ShipToLoot(Entity, Entity),
}
pub fn collision_system(
    mut world: ResMut<CollisionWorld<f32, Entity>>,
    mut cursor_selection: ResMut<CursorSelection>,
    mut collision_events: ResMut<Events<CollisionEvent>>,
    collider_types: Query<&ColliderType>,
) {
    world.update();
    cursor_selection.enemies = vec![];
    cursor_selection.loots = vec![];
    for (h1, h2, _, manifold) in world.contact_pairs(true) {
        if let Some(_tracked_contact) = manifold.deepest_contact() {
            let e1 = *world.collision_object(h1).unwrap().data();
            let e2 = *world.collision_object(h2).unwrap().data();
            let t1 = *collider_types
                .get_component::<ColliderType>(e1)
                .expect("Collision with an Entity without a ColliderType");
            let t2 = *collider_types
                .get_component::<ColliderType>(e2)
                .expect("Collision with an Entity without a ColliderType");
            if t1 == ColliderType::Missile && t2 == ColliderType::Enemy {
                collision_events.send(CollisionEvent::MissileToEnemy(e1, e2))
            }
            if t2 == ColliderType::Missile && t1 == ColliderType::Enemy {
                collision_events.send(CollisionEvent::MissileToEnemy(e2, e1))
            }
            if t1 == ColliderType::Ship && t2 == ColliderType::Loot {
                collision_events.send(CollisionEvent::ShipToLoot(e1, e2))
            }
            if t2 == ColliderType::Ship && t1 == ColliderType::Loot {
                collision_events.send(CollisionEvent::ShipToLoot(e2, e1))
            }
            if t1 == ColliderType::Cursor {
                if t2 == ColliderType::Enemy {
                    cursor_selection.loots.push(e2);
                    println!("SELECTING enemy");
                }
                if t2 == ColliderType::Loot {
                    cursor_selection.loots.push(e2);
                    println!("SELECTING loot");
                }
            }
            if t2 == ColliderType::Cursor {
                if t1 == ColliderType::Enemy {
                    cursor_selection.loots.push(e1);
                    println!("SELECTING enemy");
                }
                if t2 == ColliderType::Loot {
                    cursor_selection.loots.push(e1);
                    println!("SELECTING loot");
                }
            }
        }
    }
}

pub fn collision_event_system(
    mut commands: Commands,
    mut events: Local<EventReader<CollisionEvent>>,
    collision_events: ResMut<Events<CollisionEvent>>,
    (asset_server, audio): (Res<AssetServer>, Res<Audio>),
    mut xp_events: ResMut<Events<XpEvent>>,
    mut loot_events: ResMut<Events<LootEvent>>,
    damage_dealers: Query<&DamageDealer>,
    mut armors: Query<Mut<Armor>>,
    enemies: Query<&Enemy>,
    loots: Query<&Loot>,
    mut weapons: Query<Mut<Weapon>>,
    transforms: Query<&Transform>,
) {
    for event in events.iter(&collision_events) {
        match event {
            CollisionEvent::MissileToEnemy(e1, e2) => {
                let damage_dealer = damage_dealers.get_component::<DamageDealer>(*e1).unwrap();
                let mut armor = armors.get_component_mut::<Armor>(*e2).unwrap();
                // Multiple damaging at the same frame can happen, before despawning
                if armor.life > 0 {
                    armor.life -= damage_dealer.value;
                    println!("TOUCHED {}/{}", armor.life, armor.max_life);
                    commands.despawn_from_arena(*e1);
                    if armor.life <= 0 {
                        commands.despawn_from_arena(*e2);
                        audio.play(asset_server.load("Explosion_final.mp3"));
                        if let Ok(enemy) = enemies.get_component::<Enemy>(*e2) {
                            xp_events.send(XpEvent {
                                xp: enemy.xp,
                                source: damage_dealer.source,
                            });
                            let enemy_translation = transforms
                                .get_component::<Transform>(*e2)
                                .expect("Enemy without a transform.")
                                .translation;
                            loot_events.send(LootEvent {
                                position: enemy_translation.truncate(),
                            });
                        }
                    } else {
                        audio.play(asset_server.load("Explosion.mp3"));
                    }
                }
            }
            CollisionEvent::ShipToLoot(e1, e2) => {
                let loot = loots.get_component::<Loot>(*e2).unwrap();
                commands.despawn_from_arena(*e2);
                match loot {
                    Loot::IncreasedRateOfFire(p) => {
                        if let Ok(mut weapon) = weapons.get_component_mut::<Weapon>(*e1) {
                            weapon.fire_timer.duration =
                                weapon.fire_timer.duration / (*p as f32 / 100.0);
                        }
                    }
                    Loot::IncreasedMunitionDuration(p) => {
                        if let Ok(mut weapon) = weapons.get_component_mut::<Weapon>(*e1) {
                            weapon.munition_lifespan =
                                weapon.munition_lifespan * (*p as f32 / 100.0);
                        }
                    }
                    _ => {}
                }
                audio.play(asset_server.load("zapThreeToneUp.ogg"));
            }
        }
    }
}
