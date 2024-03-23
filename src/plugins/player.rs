use bevy::{ecs::query::Has, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_replicon::{
    client::ClientSet,
    core::replication_rules::{AppReplicationExt, Replication},
    network_event::client_event::{ClientEventAppExt, FromClient},
};
use bevy_replicon::{
    core::{common_conditions::has_authority, replicon_channels::ChannelKind},
    prelude::ClientId,
};
use bevy_xpbd_3d::{
    components::{LinearVelocity, Position, RigidBody, Rotation},
    math::{AdjustPrecision, Scalar, Vector, PI},
    plugins::{
        collision::{Collider, ColliderParent, Collisions, Sensor},
        spatial_query::{RayCaster, RayHits},
    },
    SubstepSchedule, SubstepSet,
};
use serde::{Deserialize, Serialize};

use super::{camera::fly_view, crafting::logic::Inventory, network::LocalPlayerId};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.replicate::<PlayerColor>()
            .replicate::<Player>()
            .register_type::<JumpImpulse>()
            .register_type::<SpringSettings>()
            .add_client_event::<MovePlayer>(ChannelKind::Ordered)
            .add_client_event::<RotatePlayer>(ChannelKind::Ordered)
            .add_systems(
                PreUpdate,
                (player_init_system, init_local_player).after(ClientSet::Receive),
            )
            .add_systems(
                Update,
                (
                    rotate_player.run_if(has_authority),
                    input_system.run_if(not(fly_view)),
                    (
                        update_grounded,
                        apply_gravity,
                        movement_system,
                        apply_movement_damping,
                        apply_offset,
                    )
                        .chain()
                        .run_if(has_authority),
                )
                    .chain(),
            )
            .add_systems(
                // Run collision handling in substep schedule
                SubstepSchedule,
                kinematic_controller_collisions.in_set(SubstepSet::SolveUserConstraints),
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct PlayerCollection {}

#[derive(Component, Serialize, Deserialize, PartialEq)]
pub struct Player(pub ClientId);

impl Default for Player {
    fn default() -> Self {
        Self(ClientId::SERVER)
    }
}

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub enum MovePlayer {
    Move(Vec3),
    #[default]
    Jump,
}

#[derive(Debug, Default, Deserialize, Event, Serialize)]
pub struct RotatePlayer(pub Quat);

#[derive(Component, Deserialize, Serialize, Default)]
pub struct PlayerColor(pub Color);

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct JumpImpulse(Scalar);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct SpringSettings {
    pub strength: f32,
    pub dampening: f32,
    pub height: f32,
}

impl Default for SpringSettings {
    fn default() -> Self {
        Self {
            strength: 4.0,
            dampening: 4.0,
            height: 1.0,
        }
    }
}

impl SpringSettings {
    pub fn new(strength: f32, dampening: f32, height: f32) -> Self {
        Self {
            strength,
            dampening,
            height,
        }
    }
}

/// The gravitational acceleration used for a character controller.
#[derive(Component)]
pub struct ControllerGravity(Vec3);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: RayCaster,
    gravity: ControllerGravity,
    movement: MovementBundle,
}

#[derive(Bundle)]
pub struct MovementBundle {
    spring: SpringSettings,
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub const fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
        spring_settings: SpringSettings,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
            jump_impulse: JumpImpulse(jump_impulse),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
            spring: spring_settings,
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 7.0, PI * 0.45, SpringSettings::default())
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider, gravity: Vector) -> Self {
        // Create shape caster as a slightly smaller version of collider
        // let mut caster_shape = collider.clone();
        // caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Kinematic,
            collider,
            ground_caster: RayCaster::new(Vec3::ZERO, Direction3d::NEG_Y)
                .with_max_time_of_impact(1.5),
            gravity: ControllerGravity(gravity),
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(
            acceleration,
            damping,
            jump_impulse,
            max_slope_angle,
            SpringSettings::default(),
        );
        self
    }

    pub fn with_spring(mut self, strength: f32, dampening: f32, height: f32) -> Self {
        self.movement.spring = SpringSettings::new(strength, dampening, height);
        self
    }
}

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    pub player: Player,
    pub replication: Replication,
    pub transform: Transform,
    pub color: PlayerColor,
    pub inventory: Inventory,
}

impl PlayerBundle {
    pub fn new(client_id: ClientId, color: Color) -> Self {
        Self {
            player: Player(client_id),
            replication: Replication,
            transform: Transform::from_xyz(0.0, 40.0, 0.0),
            color: PlayerColor(color),
            inventory: Inventory::default(),
        }
    }
}

#[derive(Component, Default)]
pub struct PlayerProperties {}

fn player_init_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_material: ResMut<Assets<StandardMaterial>>,
    spawned_players: Query<(Entity, &PlayerColor), Added<Player>>,
) {
    for (entity, color) in &spawned_players {
        info!("PLAYER INIT");
        let mesh_handle = meshes.add(Capsule3d::new(0.4, 1.0).mesh());
        let standard_material_handle = standard_material.add(StandardMaterial {
            base_color: color.0,
            ..Default::default()
        });
        commands.entity(entity).insert((
            Name::new("Player"),
            mesh_handle,
            standard_material_handle,
            GlobalTransform::default(),
            VisibilityBundle::default(),
            CharacterControllerBundle::new(Collider::capsule(1.0, 0.4), Vector::NEG_Y * 9.81 * 2.0)
                .with_movement(30.0, 0.92, 7.0, (30.0 as Scalar).to_radians())
                .with_spring(4.0, 4.0, 1.0),
        ));
    }
}

#[derive(Component)]
pub struct LocalPLayer;

fn init_local_player(
    mut commands: Commands,
    players: Query<(Entity, &Player)>,
    local_player: Option<Res<LocalPlayerId>>,
) {
    let Some(local_player) = local_player else {
        return;
    };

    for (entity, player) in players.iter() {
        if player.0 == local_player.0 {
            commands.entity(entity).insert(LocalPLayer);
        }
    }
}

fn rotate_player(
    mut event: EventReader<FromClient<RotatePlayer>>,
    mut players: Query<(&mut Transform, &Player)>,
) {
    for FromClient { client_id, event } in event.read() {
        for (mut transform, player) in &mut players {
            if player.0 == *client_id {
                transform.rotation = event.0;
            }
        }
    }
}

fn input_system(
    mut move_events: EventWriter<MovePlayer>,
    input: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<LocalPLayer>>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::KeyD) {
        direction += *player_transform.right();
    }
    if input.pressed(KeyCode::KeyA) {
        direction += *player_transform.left();
    }
    if input.pressed(KeyCode::KeyW) {
        println!("W");
        direction += *player_transform.forward();
    }
    if input.pressed(KeyCode::KeyS) {
        println!("S");
        direction += *player_transform.back();
    }

    if direction != Vec3::ZERO {
        move_events.send(MovePlayer::Move(direction.normalize_or_zero()));
    }

    if input.just_pressed(KeyCode::Space) {
        println!("Jump");
        move_events.send(MovePlayer::Jump);
    }
}

fn movement_system(
    time: Res<Time>,
    mut move_events: EventReader<FromClient<MovePlayer>>,
    // mut players: Query<(&Player, &mut LinearVelocity)>,
    mut controllers: Query<(
        &Player,
        &MovementAcceleration,
        &JumpImpulse,
        &mut LinearVelocity,
        Has<Grounded>,
    )>,
) {
    for FromClient { client_id, event } in move_events.read() {
        for (player, movement_acceleration, jump_impulse, mut linear_velocity, is_grounded) in
            &mut controllers
        {
            if *client_id == player.0 {
                match event {
                    MovePlayer::Move(direction) => {
                        linear_velocity.0 +=
                            *direction * movement_acceleration.0 * time.delta_seconds();
                    }
                    MovePlayer::Jump => {
                        if is_grounded {
                            linear_velocity.y = jump_impulse.0;
                        }
                    }
                }
            }
        }
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &RayCaster,
            &RayHits,
            &Rotation,
            Option<&MaxSlopeAngle>,
        ),
        With<CharacterController>,
    >,
) {
    for (entity, _ray, hits, _rotation, max_slope_angle) in &mut query {
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                hit.normal.angle_between(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn apply_gravity(
    time: Res<Time>,
    mut controllers: Query<(&ControllerGravity, &mut LinearVelocity, Has<Grounded>)>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for (gravity, mut linear_velocity, is_grounded) in &mut controllers {
        if !is_grounded {
            linear_velocity.0 += gravity.0 * delta_time;
        }
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= damping_factor.0;
        linear_velocity.z *= damping_factor.0;
    }
}

fn apply_offset(
    time: Res<Time>,
    mut character_controllers: Query<
        (&RayCaster, &RayHits, &SpringSettings, &mut LinearVelocity),
        With<CharacterController>,
    >,
) {
    for (ray, hits, spring_settings, mut velocity) in &mut character_controllers {
        for hit in hits.iter() {
            let offset = hit.time_of_impact - spring_settings.height;

            let vel = velocity.0.dot(*ray.direction);
            let force = (offset * spring_settings.strength) - (vel * spring_settings.dampening);
            velocity.0 += *ray.direction * (force * time.delta_seconds());
        }
    }
}

fn kinematic_controller_collisions(
    collisions: Res<Collisions>,
    collider_parents: Query<&ColliderParent, Without<Sensor>>,
    mut character_controllers: Query<
        (
            &RigidBody,
            &mut Position,
            &Rotation,
            &mut LinearVelocity,
            Option<&MaxSlopeAngle>,
        ),
        With<CharacterController>,
    >,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // If the collision didn't happen during this substep, skip the collision
        if !contacts.during_current_substep {
            continue;
        }

        // Get the rigid body entities of the colliders (colliders could be children)
        let Ok([collider_parent1, collider_parent2]) =
            collider_parents.get_many([contacts.entity1, contacts.entity2])
        else {
            continue;
        };

        // Get the body of the character controller and whether it is the first
        // or second entity in the collision.
        let is_first: bool;
        let (rb, mut position, rotation, mut linear_velocity, max_slope_angle) =
            if let Ok(character) = character_controllers.get_mut(collider_parent1.get()) {
                is_first = true;
                character
            } else if let Ok(character) = character_controllers.get_mut(collider_parent2.get()) {
                is_first = false;
                character
            } else {
                continue;
            };

        // This system only handles collision response for kinematic character controllers
        if !rb.is_kinematic() {
            continue;
        }

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.global_normal1(rotation)
            } else {
                -manifold.global_normal2(rotation)
            };

            // Solve each penetrating contact in the manifold
            for contact in manifold.contacts.iter().filter(|c| c.penetration > 0.0) {
                position.0 += normal * contact.penetration;
            }

            // If the slope isn't too steep to walk on but the character
            // is falling, reset vertical velocity.
            if max_slope_angle.is_some_and(|angle| normal.angle_between(Vector::Y).abs() <= angle.0)
                && linear_velocity.y < 0.0
            {
                linear_velocity.y = linear_velocity.y.max(0.0);
            }
        }
    }
}
