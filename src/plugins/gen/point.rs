use bevy::{
    ecs::{bundle::Bundle, component::Component, reflect::ReflectComponent},
    math::IVec3,
    reflect::Reflect,
    transform::components::{GlobalTransform, Transform},
};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Point(pub f32);

#[derive(PartialEq, Eq, Hash, Reflect)]
pub struct ChunkRelativePointPosition {
    chunk: IVec3,
    point_position: IVec3,
}

impl ChunkRelativePointPosition {
    pub fn new(chunk: IVec3, point_position: IVec3) -> Self {
        Self {
            chunk,
            point_position,
        }
    }
}

#[derive(Bundle, Default)]
pub struct PointBundle {
    pub point: Point,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
