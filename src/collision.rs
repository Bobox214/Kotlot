use bevy::prelude::*;
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
    mut query: Query<(&Transform, &CollisionObjectSlabHandle)>,
) {
    for (transform, &handle) in &mut query.iter() {
        let collision_object = world.get_mut(handle).unwrap();
        collision_object.set_position(Isometry2::new(
            Vector2::new(transform.translation.x(), transform.translation.y()),
            na::zero(),
        ));
    }
}

pub fn collision_system(
    mut world: ResMut<CollisionWorld<f32, Entity>>,
    mut transforms: Query<(Entity, Mut<Transform>)>,
) {
    world.update();
    for (h1, h2, _, manifold) in world.contact_pairs(true) {
        if let Some(tracked_contact) = manifold.deepest_contact() {
            let contact = tracked_contact.contact;
            let contact_normal = contact.normal.into_inner();
            let entity1 = *world.collision_object(h1).unwrap().data();
            let entity2 = *world.collision_object(h2).unwrap().data();
            println!("Collision detected!");
        }
    }
}
