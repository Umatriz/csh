use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{Has, With},
        reflect::{ReflectComponent, ReflectResource},
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    gizmos::{gizmos::Gizmos, primitives::dim3::GizmoPrimitive3d},
    math::{primitives::Cuboid, IVec3, Quat, Vec3},
    reflect::Reflect,
    render::color::Color,
    transform::components::{GlobalTransform, Transform},
    utils::HashMap,
};

use crate::GameState;

use super::point::{ChunkRelativePointPosition, Point, PointBundle};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Chunk {
    points: Vec<Entity>,
}

#[derive(Component, Reflect, Default, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct ChunkPosition(pub IVec3);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Visible;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ChunksRenderer;

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

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct CreatedChunks(pub HashMap<IVec3, Entity>);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct CreatedPoint;

pub fn create_chunk(
    commands: &mut Commands,
    created_points: &mut HashMap<ChunkRelativePointPosition, Entity>,
    size: f32,
    points_per_side: u32,
    position: IVec3,
    visible: bool,
) -> Entity {
    let real_chunk_position = position.as_vec3() * size;
    let corner = (position.as_vec3() - Vec3::ONE) * size;

    let mut points = Vec::new();

    let increment = size / (points_per_side - 1) as f32;
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
                        let point_position = corner + point_position.as_vec3() * increment;
                        let mut entity = commands.spawn(PointBundle {
                            transform: Transform::from_xyz(
                                point_position.x,
                                point_position.y,
                                point_position.z,
                            ),
                            ..Default::default()
                        });

                        if x == 0
                            || y == 0
                            || z == 0
                            || x == (points_per_side - 1)
                            || y == (points_per_side - 1)
                            || z == (points_per_side - 1)
                        {
                            created_points
                                .insert(point_relative_position, entity.insert(CreatedPoint).id());
                        }

                        points.push(entity.id());
                    }
                }
            }
        }
    }

    let mut chunk = commands.spawn(ChunkBundle {
        chunk: Chunk::new(points),
        chunk_position: ChunkPosition(position),
        transform: Transform::from_xyz(
            real_chunk_position.x,
            real_chunk_position.y,
            real_chunk_position.z,
        ),
        ..Default::default()
    });

    if visible {
        chunk.insert(Visible);
    }

    chunk.id()
}

#[derive(Reflect, Resource, Clone)]
pub struct ChunkingPluginSettings {
    pub chunk_size: f32,
    pub points_per_side: u32,
    pub chunks_visible: u8,
}

impl Default for ChunkingPluginSettings {
    fn default() -> Self {
        Self {
            chunk_size: 10.0,
            points_per_side: 5,
            chunks_visible: 1,
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
            .register_type::<Chunk>()
            .register_type::<ChunkPosition>()
            .init_resource::<CreatedPoints>()
            .register_type::<CreatedPoints>()
            .init_resource::<CreatedChunks>()
            .register_type::<CreatedChunks>()
            .add_systems(
                OnEnter(GameState::Game),
                |mut commands: Commands,
                 settings: Res<ChunkingPluginSettings>,
                 mut created_points: ResMut<CreatedPoints>| {
                    create_chunk(
                        &mut commands,
                        &mut created_points.0,
                        settings.chunk_size,
                        settings.points_per_side,
                        IVec3::ZERO,
                        true,
                    );
                },
            )
            .add_systems(Update, (draw_points, draw_chunks))
            .add_systems(Update, render_chunks.run_if(in_state(GameState::Game)));
    }
}

fn render_chunks(
    mut commands: Commands,
    settings: Res<ChunkingPluginSettings>,
    mut created_points: ResMut<CreatedPoints>,
    mut created_chunks: ResMut<CreatedChunks>,
    renderer: Query<&GlobalTransform, With<ChunksRenderer>>,
) {
    for transform in renderer.iter() {
        let current_chunk = (transform.translation() / settings.chunk_size)
            .round()
            .as_ivec3();

        let chunks_visible = settings.chunks_visible as i32;

        let mut viewed_chunks = vec![];

        for x_offset in -chunks_visible..=chunks_visible {
            for y_offset in -chunks_visible..=chunks_visible {
                for z_offset in -chunks_visible..=chunks_visible {
                    let viewed_chunk = IVec3::new(
                        current_chunk.x + x_offset,
                        current_chunk.y + y_offset,
                        current_chunk.z + z_offset,
                    );

                    viewed_chunks.push(viewed_chunk);

                    match created_chunks.0.get(&viewed_chunk) {
                        Some(entity) => {
                            commands.entity(*entity).insert(Visible);
                        }
                        None => {
                            let entity = create_chunk(
                                &mut commands,
                                &mut created_points.0,
                                settings.chunk_size,
                                settings.points_per_side,
                                viewed_chunk,
                                true,
                            );

                            created_chunks.0.insert(viewed_chunk, entity);
                        }
                    }
                }
            }
        }

        dbg!(viewed_chunks.len());

        created_chunks
            .0
            .iter()
            .filter(|(position, _)| !viewed_chunks.contains(position))
            .for_each(|(_, entity)| {
                commands.entity(*entity).remove::<Visible>();
            });
    }
}

fn draw_points(mut gizmos: Gizmos, points: Query<(&Transform, &Point, Has<CreatedPoint>)>) {
    // for (transform, point, is_created) in points.iter() {
    //     let color = if is_created {
    //         Color::GREEN
    //     } else {
    //         color_lerp(Color::BLACK, Color::WHITE, point.0)
    //     };
    //     gizmos.sphere(transform.translation, Quat::IDENTITY, 0.05, color);
    // }
}

fn draw_chunks(
    mut gizmos: Gizmos,
    settings: Res<ChunkingPluginSettings>,
    chunks: Query<(&Transform, Has<Visible>), With<Chunk>>,
) {
    for (transform, is_visible) in chunks.iter() {
        let color = if is_visible {
            Color::YELLOW
        } else {
            Color::RED
        };
        gizmos.primitive_3d(
            Cuboid::new(
                settings.chunk_size,
                settings.chunk_size,
                settings.chunk_size,
            ),
            transform.translation,
            Quat::IDENTITY,
            color,
        )
    }
}
