use super::*;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Cardinal {
    NE,
    N,
    NW,
    W,
    SW,
    S,
    SE,
    E,
}
pub struct SpriteGhost {
    /// 0, 1 or 2 for a 3 ghost arena
    id: u8,
    /// Parent scale factor, to inverse it when defining translations
    parent_scale: Vec3,
}

/// Tag component for Entity with SpriteGhost children
struct SpriteGhostParent {}
pub trait SpawnWithGhosts {
    fn spawn_with_ghosts(&mut self, sprite_components: SpriteComponents) -> &mut Self;
}
impl SpawnWithGhosts for Commands {
    ///! Spawn the SpriteComponents, and 3 ghosts child with the same material
    ///! Ghost 'translation' and rotation will be kept in sync,
    fn spawn_with_ghosts(&mut self, sprite_components: SpriteComponents) -> &mut Self {
        let material = sprite_components.material.clone();
        let parent_scale = sprite_components.transform.scale();
        self.spawn(sprite_components)
            .with(SpriteGhostParent {})
            .with_children(|parent| {
                for id in 0..3 {
                    parent
                        .spawn(SpriteComponents {
                            material,
                            ..Default::default()
                        })
                        .with(SpriteGhost { id, parent_scale });
                }
            })
    }
}

pub struct Arena {
    pub size: Vec2,
    /// Quarter of the arena currently shown
    /// Only one of NE,NW,SE,SW
    /// Used for ghosts
    pub shown: Cardinal,
}

pub fn spriteghost_quadrant_system(
    arena: ChangedRes<Arena>,
    mut query: Query<(&SpriteGhost, Mut<Transform>)>,
) {
    for (ghost, mut transform) in &mut query.iter() {
        let (dx, dy) = match (arena.shown, ghost.id) {
            (Cardinal::NW, 0) => (-arena.size.x(), 0.0),
            (Cardinal::NW, 1) => (-arena.size.x(), arena.size.y()),
            (Cardinal::NW, 2) => (0.0, arena.size.y()),
            (Cardinal::NE, 0) => (0.0, arena.size.y()),
            (Cardinal::NE, 1) => (arena.size.x(), arena.size.y()),
            (Cardinal::NE, 2) => (arena.size.x(), 0.0),
            (Cardinal::SE, 0) => (arena.size.x(), 0.0),
            (Cardinal::SE, 1) => (arena.size.x(), -arena.size.y()),
            (Cardinal::SE, 2) => (0.0, -arena.size.y()),
            (Cardinal::SW, 0) => (0.0, -arena.size.y()),
            (Cardinal::SW, 1) => (-arena.size.x(), -arena.size.y()),
            (Cardinal::SW, 2) => (-arena.size.x(), 0.0),
            _ => panic!("Unexpected arena.shown,ghost.id combination"),
        };
        *(transform.translation_mut().x_mut()) = dx / ghost.parent_scale.x();
        *(transform.translation_mut().y_mut()) = dy / ghost.parent_scale.y();
    }
}

/// Keep ghost SpriteComponents in sync with Parent SpriteComponents changes.
/// Transform is already local, sync is already handled by Bevy.
pub fn spriteghost_sync_system() {}

pub fn position_system(
    time: Res<Time>,
    arena: Res<Arena>,
    mut query: Query<(Mut<Transform>, Mut<Velocity>)>,
) {
    let elapsed = time.delta_seconds;
    for (mut transform, mut velocity) in &mut query.iter() {
        let translation = transform.translation_mut();
        *translation.x_mut() += velocity.0.x() * elapsed;
        *translation.y_mut() += velocity.0.y() * elapsed;
        velocity.0 = velocity.0 * 0.1f32.powf(time.delta_seconds);

        let half_width = arena.size.x() / 2.0;
        let half_height = arena.size.y() / 2.0;
        // Wrap around the world, as a torus.
        if translation.x() < -half_width && velocity.0.x() < 0.0 {
            *translation.x_mut() = half_width;
        } else if translation.x() > half_width && velocity.0.x() > 0.0 {
            *translation.x_mut() = -half_width;
        }
        if translation.y() < -half_height && velocity.0.y() < 0.0 {
            *translation.y_mut() = half_height;
        } else if translation.y() > half_height && velocity.0.y() > 0.0 {
            *translation.y_mut() = -half_height;
        }
    }
}
