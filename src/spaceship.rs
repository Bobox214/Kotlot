use super::*;
pub struct Spaceship {
    pub max_angvel: f32,
    pub max_linvel: f32,
    pub max_latvel: f32,
}
pub struct Weapon {
    pub fire_timer: Timer,
    pub munition_lifespan: f32,
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

pub struct XpEvent {
    pub xp: u32,
    pub source: Entity,
}
pub struct Progression {
    pub level: u32,
    pub xp: u32,
}

const XP_MAX_LEVEL: u32 = 6;
const XP_PER_LEVEL: &[u32] = &[0, 10, 30, 100, 300, 1000, 0];

impl Progression {
    pub fn new() -> Progression {
        Progression { level: 1, xp: 0 }
    }
    pub fn add_xp(&mut self, xp: u32) {
        if self.level == XP_MAX_LEVEL {
            return;
        }
        self.xp += xp;
        if self.xp >= XP_PER_LEVEL[self.level as usize] {
            self.xp -= XP_PER_LEVEL[self.level as usize];
            self.level += 1;
        }
        println!(
            "Level {} ; {}/{}",
            self.level, self.xp, XP_PER_LEVEL[self.level as usize]
        );
    }
}

pub fn xp_system(
    mut xp_event_reader: Local<EventReader<XpEvent>>,
    xp_events: Res<Events<XpEvent>>,
    mut progressions: Query<Mut<Progression>>,
) {
    for event in xp_event_reader.iter(&*xp_events) {
        if let Ok(mut progression) = progressions.get_component_mut::<Progression>(event.source) {
            progression.add_xp(event.xp);
        }
    }
}

pub fn spawn_player_spaceship(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut collide_world: ResMut<CollisionWorld<f32, Entity>>,
    collide_groups: Res<CollideGroups>,
    cameras: Query<(Entity, &Camera)>,
) {
    let camera_entity = cameras
        .iter()
        .filter_map(|(entity, camera)| {
            if camera.name == Some(String::from("Camera2d")) {
                Some(entity)
            } else {
                None
            }
        })
        .next()
        .unwrap();
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
            fire_timer: Timer::from_seconds(1.0, false),
            munition_lifespan: 0.5,
        })
        .with(Progression::new())
        .with(ColliderType::Ship);
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
}
