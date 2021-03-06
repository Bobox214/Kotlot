use bevy_contrib_bobox::{OutlineConfiguration, OutlineMaterial};

use super::*;
pub struct LootEvent {
    pub position: Vec2,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Loot {
    IncreasedRateOfFire(u32),
    IncreasedMunitionDuration(u32),
    None,
}
impl Loot {
    fn get_asset(&self) -> String {
        match &self {
            Loot::IncreasedRateOfFire(_) => String::from("bolt_silver.png"),
            Loot::IncreasedMunitionDuration(_) => String::from("bolt_bronze.png"),
            Loot::None => panic!("Can't get asset for Loot::None"),
        }
    }
}

pub fn loot_spawn_system(
    mut commands: Commands,
    mut loot_event_reader: Local<EventReader<LootEvent>>,
    loot_events: Res<Events<LootEvent>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut outline_materials: ResMut<Assets<OutlineMaterial>>,
    (mut collide_world, collide_groups): (ResMut<CollisionWorld<f32, Entity>>, Res<CollideGroups>),
) {
    for event in loot_event_reader.iter(&*loot_events) {
        let mut rng = thread_rng();
        let loot = match rng.gen_range(0, 3) {
            0 => Loot::IncreasedRateOfFire(200),
            1 => Loot::IncreasedMunitionDuration(150),
            _ => Loot::None,
        };
        if loot != Loot::None {
            commands
                .spawn_with_ghosts(SpriteComponents {
                    transform: Transform {
                        translation: Vec3::new(event.position.x(), event.position.y(), -0.2),
                        scale: Vec3::splat(0.5),
                        ..Default::default()
                    },
                    material: materials.add(asset_server.load(loot.get_asset().as_str()).into()),
                    ..Default::default()
                })
                .with(loot)
                .with(ColliderType::Loot)
                .with(TweenScale::new(Vec3::splat(0.4), Vec3::splat(0.75), 1.0))
                .with(outline_materials.add(OutlineMaterial {
                    configuration: OutlineConfiguration {
                        color: Color::rgb(0.7, 0.7, 1.0),
                        width: 5,
                        ..Default::default()
                    },
                    with_outline: false,
                }));
            let entity = commands.current_entity().unwrap();
            let shape = ShapeHandle::new(Ball::new(30.0 * 0.75 * 0.5));
            let (collision_object_handle, _) = collide_world.add(
                Isometry2::new(
                    Vector2::new(event.position.x(), event.position.y()),
                    na::zero(),
                ),
                shape,
                collide_groups.loots,
                GeometricQueryType::Contacts(0.0, 0.0),
                entity,
            );
            commands.insert(entity, (collision_object_handle,));
        }
    }
}

pub struct TweenScale {
    pub min: Vec3,
    pub max: Vec3,
    pub period: f32,
    pub increase: bool,
    pub rate: Vec3,
}
impl TweenScale {
    pub fn new(min: Vec3, max: Vec3, period: f32) -> TweenScale {
        TweenScale {
            min,
            max,
            period,
            increase: true,
            rate: (max - min) / (period / 2.0),
        }
    }
}

pub fn tweenscale_system(time: Res<Time>, mut query: Query<(Mut<Transform>, Mut<TweenScale>)>) {
    for (mut transform, mut tweenscale) in query.iter_mut() {
        let diff = tweenscale.rate * time.delta_seconds;
        if tweenscale.increase {
            transform.scale += diff;
            if transform.scale > tweenscale.max {
                tweenscale.increase = false;
            }
        } else {
            transform.scale -= diff;
            if transform.scale < tweenscale.min {
                tweenscale.increase = true;
            }
        }
    }
}
