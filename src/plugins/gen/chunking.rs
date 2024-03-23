use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::With,
        reflect::{ReflectComponent, ReflectResource},
        schedule::OnEnter,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    gizmos::gizmos::Gizmos,
    math::{IVec3, Quat, Vec3},
    reflect::Reflect,
    render::{color::Color, primitives::Aabb},
    transform::components::{GlobalTransform, Transform},
    utils::HashMap,
};

use crate::{utils::color_lerp, GameState};

use super::point::{ChunkRelativePointPosition, Point, PointBundle};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Chunk {
    points: Vec<Entity>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ChunkPosition(pub IVec3);

impl Chunk {
    pub fn new(points: Vec<Entity>) -> Self {
        Self { points }
    }

    pub fn points_as_slice(&self) -> &[Entity] {
        &self.points
    }

    pub fn iter_points(&self) -> std::slice::Iter<'_, Entity> {
        self.points_as_slice().iter()
    }
}

#[derive(Bundle, Default)]
pub struct ChunkBundle {
    pub chunk: Chunk,
    pub chunk_position: ChunkPosition,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct CreatedPoints(pub HashMap<ChunkRelativePointPosition, Entity>);

pub fn create_chunk(
    commands: &mut Commands,
    size: f32,
    position: IVec3,
    points_per_side: u32,
    created_points: &mut HashMap<ChunkRelativePointPosition, Entity>,
) {
    let real_chunk_position = (position.as_vec3() * size - Vec3::splat(size / 2.0));
    let aabb = Aabb {
        center: real_chunk_position.into(),
        half_extents: Vec3::splat(size / 2.0).into(),
    };

    let mut points = Vec::new();

    let increment = size / points_per_side as f32;
    let points_per_side = points_per_side as i32;
    for x in 0..points_per_side {
        for y in 0..points_per_side {
            for z in 0..points_per_side {
                let point_position = IVec3::new(x, y, z);
                let point_relative_position =
                    ChunkRelativePointPosition::new(position, point_position);

                match created_points.get(&point_relative_position) {
                    Some(entity) => points.push(*entity),
                    None => {
                        let point_position = point_position.as_vec3() * increment;
                        let entity = commands
                            .spawn(PointBundle {
                                transform: Transform::from_xyz(
                                    point_position.x,
                                    point_position.y,
                                    point_position.z,
                                ),
                                ..Default::default()
                            })
                            .id();
                        created_points.insert(point_relative_position, entity);
                        points.push(entity);
                    }
                }
            }
        }
    }

    commands.spawn(ChunkBundle {
        chunk: Chunk::new(points),
        chunk_position: ChunkPosition(position),
        transform: Transform::from_xyz(
            real_chunk_position.x,
            real_chunk_position.y,
            real_chunk_position.z,
        ),
        ..Default::default()
    });
}

#[derive(Reflect, Resource, Clone)]
pub struct ChunkingPluginSettings {
    pub chunk_size: f32,
    pub points_per_side: u32,
}

impl Default for ChunkingPluginSettings {
    fn default() -> Self {
        Self {
            chunk_size: 10.0,
            points_per_side: 5,
        }
    }
}

#[derive(Default)]
pub struct ChunkingPlugin {
    pub settings: ChunkingPluginSettings,
}

impl Plugin for ChunkingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings.clone())
            .init_resource::<CreatedPoints>()
            .add_systems(Update, draw_points)
            .add_systems(OnEnter(GameState::Game), create_chunk_system);
    }
}

fn create_chunk_system(
    mut commands: Commands,
    settings: Res<ChunkingPluginSettings>,
    mut created_points: ResMut<CreatedPoints>,
) {
    create_chunk(
        &mut commands,
        settings.chunk_size,
        IVec3::new(1, 1, 1),
        settings.points_per_side,
        &mut created_points.0,
    )
}

fn draw_points(mut gizmos: Gizmos, points: Query<(&Transform, &Point)>) {
    for (transform, point) in points.iter() {
        gizmos.sphere(
            transform.translation,
            Quat::IDENTITY,
            0.05,
            color_lerp(Color::BLACK, Color::WHITE, point.0),
        );
    }
}
