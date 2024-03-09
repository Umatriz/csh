// use bevy::{
//     app::{Plugin, Update},
//     asset::Handle,
//     core::Name,
//     ecs::{
//         bundle::Bundle,
//         component::Component,
//         entity::Entity,
//         event::{Event, EventReader, EventWriter},
//         query::{Added, With},
//         schedule::{common_conditions::in_state, IntoSystemConfigs},
//         system::{Commands, Query, Res},
//         world::EntityWorldMut,
//     },
//     gizmos::gizmos::Gizmos,
//     math::{Vec2, Vec3, Vec3Swizzles},
//     render::{
//         camera::Camera,
//         color::Color,
//         texture::Image,
//         view::{InheritedVisibility, ViewVisibility, Visibility, VisibilityBundle},
//     },
//     sprite::Sprite,
//     time::Time,
//     transform::components::{GlobalTransform, Transform},
//     window::{PrimaryWindow, Window},
// };
// use bevy_mod_picking::pointer::PointerLocation;
// use bevy_replicon::{
//     core::replicon_channels::ChannelKind,
//     network_event::client_event::{ClientEventAppExt, FromClient},
//     prelude::{has_authority, AppReplicationExt, ClientId, Replication},
//     server::ClientEntityMap,
// };
// use bevy_xpbd_2d::{
//     components::{
//         AngularVelocity, CoefficientCombine, ExternalForce, LinearVelocity, Restitution, RigidBody,
//     },
//     plugins::collision::Collider,
// };
// use ndarray::{Array1, Array2};
// use serde::{Deserialize, Serialize};

// use crate::GameState;

// pub struct CursorPlugin;

// impl Plugin for CursorPlugin {
//     fn build(&self, app: &mut bevy::prelude::App) {
//         app.replicate::<CursorColor>()
//             .replicate::<Cursor>()
//             .replicate::<CursorParticle>()
//             .add_client_event::<CursorMoveEvent>(ChannelKind::Ordered)
//             .add_systems(
//                 Update,
//                 (
//                     cursor_init,
//                     (
//                         cursor_input,
//                         draw_velocity_vector,
//                         handle_particle_collision,
//                         particle_movement,
//                         // particle_separation,
//                     )
//                         .run_if(in_state(GameState::Game)),
//                     cursor_movement_event.run_if(has_authority),
//                 ),
//             );
//     }
// }

// const PARTICLE_SPEED: f32 = 0.5;

// #[derive(Bundle)]
// pub struct CursorBundle {
//     pub cursor: Cursor,
//     pub color: CursorColor,
//     pub transform: Transform,
//     pub replication: Replication,
// }

// #[derive(Component, Serialize, Deserialize)]
// pub struct CursorColor(pub Color);

// #[derive(Component, Serialize, Deserialize)]
// pub struct Cursor(pub ClientId);

// #[derive(Component, Serialize, Deserialize, Clone)]
// pub struct CursorParticle {
//     pub parent: Entity,
// }

// // impl ClientEntityMap for CursorParticle {
// //     // fn map_entities<T: bevy_replicon::prelude::Mapper>(&mut self, mapper: &mut T) {
// //     //     self.parent = mapper.map(self.parent)
// //     // }
// // }

// #[derive(Event, Serialize, Deserialize)]
// pub struct CursorMoveEvent(pub Vec2);

// fn cursor_init(mut commands: Commands, spawned_players: Query<Entity, Added<Cursor>>) {
//     for entity in &spawned_players {
//         commands
//             .entity(entity)
//             .insert((
//                 GlobalTransform::default(),
//                 VisibilityBundle::default(),
//                 Handle::<Image>::default(),
//             ))
//             .add(|mut c: EntityWorldMut<'_>| {
//                 if let Some(color) = c.get::<CursorColor>() {
//                     c.insert(Sprite {
//                         color: color.0,
//                         custom_size: Some(Vec2::splat(30.0)),
//                         ..Default::default()
//                     });
//                 }
//             });

//         commands.spawn_batch(Array2::from_shape_fn((5, 1), |(i, j)| {
//             (
//                 Name::new("Particle"),
//                 Replication,
//                 CursorParticle { parent: entity },
//                 Transform::from_xyz(i as f32 * 50.0, j as f32, 0.0),
//                 GlobalTransform::default(),
//                 Visibility::default(),
//                 InheritedVisibility::default(),
//                 ViewVisibility::default(),
//                 Handle::<Image>::default(),
//                 RigidBody::Dynamic,
//                 ExternalForce::default().with_persistence(true),
//                 Collider::capsule(1.0, 15.0),
//                 Sprite {
//                     color: Color::RED,
//                     custom_size: Some(Vec2::splat(15.0)),
//                     ..Default::default()
//                 },
//             )
//         }));
//     }
// }

// fn cursor_movement_event(
//     mut move_events: EventReader<FromClient<CursorMoveEvent>>,
//     mut cursors: Query<(&Cursor, &mut Transform)>,
// ) {
//     for FromClient { client_id, event } in move_events.read() {
//         for (cursor, mut cursor_transform) in cursors.iter_mut() {
//             if *client_id == cursor.0 {
//                 let position = cursor_transform.translation.lerp(event.0.extend(0.0), 0.5);
//                 cursor_transform.translation = position;
//             }
//         }
//     }
// }

// fn cursor_input(
//     camera: Query<(&Camera, &GlobalTransform)>,
//     // window: Query<&Window, With<PrimaryWindow>>,
//     pointer: Query<&PointerLocation>,
//     mut move_events: EventWriter<CursorMoveEvent>,
// ) {
//     // let window = window.get_single().unwrap();
//     let (camera, camera_transform) = camera.get_single().unwrap();
//     let pointer = pointer.get_single().unwrap();

//     if let Some(world_position) = pointer
//         .location()
//         .and_then(|location| camera.viewport_to_world(camera_transform, location.position))
//         .map(|ray| ray.origin.truncate())
//     {
//         move_events.send(CursorMoveEvent(world_position));
//     }
// }

// fn particle_movement(
//     cursors: Query<&GlobalTransform, With<Cursor>>,
//     mut particles: Query<(&mut LinearVelocity, &Transform, &CursorParticle)>,
//     time: Res<Time>,
// ) {
//     for (mut linear_velocity, transform, particle) in particles.iter_mut() {
//         if let Ok(parent_transform) = cursors.get(particle.parent) {
//             let raw_direction = (transform.translation - parent_transform.translation()).xy();
//             let distance = raw_direction.length();

//             let offset = 50.0 - distance;
//             let direction = raw_direction.normalize();

//             let vel = linear_velocity.0.dot(direction);

//             let strength = 100.0;
//             let dampening = 70.0;
//             let force = (offset * strength) - (vel * dampening);
//             linear_velocity.0 += direction * (force * time.delta_seconds());
//         }
//     }
// }

// fn draw_velocity_vector(
//     velocity_transform: Query<(&Transform, &CursorParticle)>,
//     parents: Query<&Transform, With<Cursor>>,
//     mut gizmos: Gizmos,
// ) {
//     for (transform, particle) in velocity_transform.iter() {
//         if let Ok(parent_transform) = parents.get(particle.parent) {
//             gizmos.line_2d(
//                 transform.translation.xy(),
//                 parent_transform.translation.xy(),
//                 Color::GREEN,
//             )
//         }
//     }
// }

// fn handle_particle_collision() {}

// fn particle_separation(
//     particles: Query<Entity, With<CursorParticle>>,
//     mut transforms: Query<&mut Transform>,
// ) {
//     for entity in particles.iter() {
//         let current_transform = transforms.get(entity).unwrap();
//         let mut separation = Vec3::ZERO;
//         let mut separation_count = 0;

//         for other in particles.iter().filter(|e| *e != entity) {
//             let other_transform = transforms.get(other).unwrap();
//             if (current_transform.translation - other_transform.translation).length() < 5.0 {
//                 separation += (current_transform.translation - other_transform.translation)
//                     / (current_transform.translation - other_transform.translation).length();

//                 separation_count += 1;
//             }
//         }

//         if separation_count > 0 {
//             separation /= separation_count as f32;
//         }

//         let mut current_transform_mut = transforms.get_mut(entity).unwrap();
//         current_transform_mut.translation = separation;
//     }
// }
