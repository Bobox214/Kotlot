use super::*;
use bevy::ecs::Command;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum ArenaQuadrant {
    NE,
    NW,
    SW,
    SE,
}
pub struct SpriteGhost {
    /// Parent entity to get the transform
    pub parent: Entity,
    /// 0, 1 or 2 for a 3 ghost arena
    pub id: u8,
}

/// Tag component for Entity with SpriteGhost children
pub struct SpriteGhostChildren(pub Vec<Entity>);
struct DespawnWithGhosts {
    entity: Entity,
}

impl Command for DespawnWithGhosts {
    fn write(self: Box<Self>, world: &mut World, _resources: &mut Resources) {
        if let Ok(children) = world.get::<SpriteGhostChildren>(self.entity) {
            for &entity in children.0.clone().iter() {
                if let Err(e) = world.despawn(entity) {
                    println!("Failed to despawn ghost entity {:?}: {}", entity.id(), e);
                }
            }
        }
        if let Err(e) = world.despawn(self.entity) {
            println!(
                "Failed to despawn main entity {:?}: {}",
                self.entity.id(),
                e
            );
        }
    }
}
pub trait SpriteGhostExt {
    fn spawn_with_ghosts(&mut self, sprite_components: SpriteComponents) -> &mut Self;
    fn despawn_with_ghosts(&mut self, entity: Entity) -> &mut Self;
}
impl SpriteGhostExt for Commands {
    ///! Spawn the SpriteComponents, and 3 ghosts child with the same material
    ///! Ghost 'translation' and rotation will be kept in sync,
    fn spawn_with_ghosts(&mut self, sprite_components: SpriteComponents) -> &mut Self {
        let material = sprite_components.material.clone();
        {
            let mut commands = self.commands.lock();
            commands.spawn(sprite_components);
            let parent = commands.current_entity.unwrap();
            let child_ids = (0..3)
                .map(|id| {
                    commands
                        .spawn(SpriteComponents {
                            material: material.clone(),
                            ..Default::default()
                        })
                        .with(SpriteGhost { parent, id });
                    commands.current_entity.unwrap()
                })
                .collect::<Vec<_>>();
            commands.current_entity = Some(parent);
            commands.with(SpriteGhostChildren(child_ids));
        }
        self
    }
    fn despawn_with_ghosts(&mut self, entity: Entity) -> &mut Self {
        self.add_command(DespawnWithGhosts { entity })
    }
}

pub struct Arena {
    pub size: Vec2,
    /// Quarter of the arena currently shown
    /// Only one of NE,NW,SE,SW
    /// Used for ghosts
    pub shown: ArenaQuadrant,
}

pub fn spriteghost_quadrant_system(
    arena: Res<Arena>,
    mut query: Query<(&SpriteGhost, Mut<Transform>)>,
    query_transform: Query<&Transform>,
) {
    for (ghost, mut transform) in &mut query.iter() {
        if let Ok(parent_transform) = query_transform.get::<Transform>(ghost.parent) {
            let translation = match (arena.shown, ghost.id) {
                (ArenaQuadrant::NW, 0) => Vec3::new(-arena.size.x(), 0.0, 0.0),
                (ArenaQuadrant::NW, 1) => Vec3::new(-arena.size.x(), arena.size.y(), 0.0),
                (ArenaQuadrant::NW, 2) => Vec3::new(0.0, arena.size.y(), 0.0),
                (ArenaQuadrant::NE, 0) => Vec3::new(0.0, arena.size.y(), 0.0),
                (ArenaQuadrant::NE, 1) => Vec3::new(arena.size.x(), arena.size.y(), 0.0),
                (ArenaQuadrant::NE, 2) => Vec3::new(arena.size.x(), 0.0, 0.0),
                (ArenaQuadrant::SE, 0) => Vec3::new(arena.size.x(), 0.0, 0.0),
                (ArenaQuadrant::SE, 1) => Vec3::new(arena.size.x(), -arena.size.y(), 0.0),
                (ArenaQuadrant::SE, 2) => Vec3::new(0.0, -arena.size.y(), 0.0),
                (ArenaQuadrant::SW, 0) => Vec3::new(0.0, -arena.size.y(), 0.0),
                (ArenaQuadrant::SW, 1) => Vec3::new(-arena.size.x(), -arena.size.y(), 0.0),
                (ArenaQuadrant::SW, 2) => Vec3::new(-arena.size.x(), 0.0, 0.0),
                _ => panic!("Unexpected arena.shown,ghost.id combination"),
            };
            transform.translation = parent_transform.translation + translation;
            transform.rotation = parent_transform.rotation;
            transform.scale = parent_transform.scale;
        }
    }
}

/// Keep ghost SpriteComponents in sync with Parent SpriteComponents changes.
/// Transform is already local, sync is already handled by Bevy.
pub fn spriteghost_sync_system() {}

pub fn position_system(
    time: Res<Time>,
    arena: Res<Arena>,
    mut query: Query<(Mut<Transform>, Mut<Movement>)>,
) {
    let elapsed = time.delta_seconds;
    for (mut transform, mut movement) in &mut query.iter() {
        transform.translation += Vec3::new(
            movement.speed.x() * elapsed,
            movement.speed.y() * elapsed,
            0.0,
        );
        movement.speed = movement.speed * movement.dampening.powf(time.delta_seconds);

        let half_width = arena.size.x() / 2.0;
        let half_height = arena.size.y() / 2.0;
        // Wrap around the world, as a torus.
        if transform.translation.x() < -half_width && movement.speed.x() < 0.0 {
            *transform.translation.x_mut() = half_width;
        } else if transform.translation.x() > half_width && movement.speed.x() > 0.0 {
            *transform.translation.x_mut() = -half_width;
        }
        if transform.translation.y() < -half_height && movement.speed.y() < 0.0 {
            *transform.translation.y_mut() = half_height;
        } else if transform.translation.y() > half_height && movement.speed.y() > 0.0 {
            *transform.translation.y_mut() = -half_height;
        }
    }
}

pub fn spawn_arena_markers(
    mut commands: Commands,
    arena: Res<Arena>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for x in 0..((arena.size.x() / 200.0) as i32) {
        for y in 0..((arena.size.y() / 200.0) as i32) {
            let color = match (x, y) {
                (0, 0) => Color::rgb(0.9, 0.9, 0.9),
                (0, _) => Color::rgb(0.7, 0.7, 0.7),
                (_, 0) => Color::rgb(0.7, 0.7, 0.7),
                _ => Color::rgb(0.5, 0.5, 0.5),
            };
            commands.spawn(SpriteComponents {
                sprite: Sprite::new(Vec2::new(2.0, 2.0)),
                material: materials.add(color.into()),
                transform: Transform {
                    translation: Vec3::new((x * 100) as f32, (y * 100) as f32, -1.0),
                    ..Default::default()
                },
                ..Default::default()
            });
            if x != 0 {
                commands.spawn(SpriteComponents {
                    sprite: Sprite::new(Vec2::new(2.0, 2.0)),
                    material: materials.add(color.into()),
                    transform: Transform {
                        translation: Vec3::new((-x * 100) as f32, (y * 100) as f32, -1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            if y != 0 {
                commands.spawn(SpriteComponents {
                    sprite: Sprite::new(Vec2::new(2.0, 2.0)),
                    material: materials.add(color.into()),
                    transform: Transform {
                        translation: Vec3::new((x * 100) as f32, (-y * 100) as f32, -1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            if y != 0 && x != 0 {
                commands.spawn(SpriteComponents {
                    sprite: Sprite::new(Vec2::new(2.0, 2.0)),
                    material: materials.add(color.into()),
                    transform: Transform {
                        translation: Vec3::new((-x * 100) as f32, (-y * 100) as f32, -1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
        }
    }
}
