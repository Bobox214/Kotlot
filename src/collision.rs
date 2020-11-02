use super::*;
use ncollide2d::{
    na::{self, Isometry2, Vector2},
    pipeline::{CollisionGroups, CollisionObjectSlabHandle},
    world::CollisionWorld,
};

pub struct CollideGroups {
    pub ships: CollisionGroups,
    pub asteroids: CollisionGroups,
    pub missiles: CollisionGroups,
}
pub fn setup_ncollide(mut commands: Commands) {
    let world = CollisionWorld::<f32, Entity>::new(0.02);
    let ships = CollisionGroups::new()
        .with_membership(&[1])
        .with_whitelist(&[1, 2, 3])
        .with_blacklist(&[]);
    let asteroids = CollisionGroups::new()
        .with_membership(&[2])
        .with_whitelist(&[1, 2, 3])
        .with_blacklist(&[]);
    let missiles = CollisionGroups::new()
        .with_membership(&[3])
        .with_whitelist(&[2])
        .with_blacklist(&[]);
    commands.insert_resource(CollideGroups {
        ships,
        asteroids,
        missiles,
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
enum CollisionEvent {
    DamageDealing(Entity, Entity),
}
pub fn collision_system(
    mut commands: Commands,
    mut world: ResMut<CollisionWorld<f32, Entity>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut xp_events: ResMut<Events<XpEvent>>,
    mut loot_events: ResMut<Events<LootEvent>>,
    damage_dealers: Query<&DamageDealer>,
    mut armors: Query<Mut<Armor>>,
    enemies: Query<Mut<Enemy>>,
    transforms: Query<&Transform>,
) {
    world.update();
    let mut events = vec![];
    for (h1, h2, _, manifold) in world.contact_pairs(true) {
        if let Some(_tracked_contact) = manifold.deepest_contact() {
            let e1 = *world.collision_object(h1).unwrap().data();
            let e2 = *world.collision_object(h2).unwrap().data();
            if damage_dealers.get_component::<DamageDealer>(e1).is_ok()
                && armors.get_component::<Armor>(e2).is_ok()
            {
                events.push(CollisionEvent::DamageDealing(e1, e2))
            }
            if damage_dealers.get_component::<DamageDealer>(e2).is_ok()
                && armors.get_component::<Armor>(e1).is_ok()
            {
                events.push(CollisionEvent::DamageDealing(e2, e1))
            }
        }
    }
    for event in events.iter() {
        match event {
            CollisionEvent::DamageDealing(e1, e2) => {
                let damage_dealer = damage_dealers.get_component::<DamageDealer>(*e1).unwrap();
                let mut armor = armors.get_component_mut::<Armor>(*e2).unwrap();
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
    }
}
