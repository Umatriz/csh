use bevy::{
    app::App,
    asset::Assets,
    ecs::{
        schedule::OnEnter,
        system::{Commands, ResMut},
    },
    math::{primitives::Plane3d, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    render::{
        color::Color,
        mesh::{Mesh, Meshable},
    },
    transform::components::Transform,
};
use bevy_xpbd_3d::{components::RigidBody, plugins::collision::Collider};

use crate::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), spawn_floor);
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = Plane3d::new(Vec3::Y).mesh().size(50.0, 50.0).build();
    commands.spawn((
        RigidBody::Static,
        Collider::convex_hull_from_mesh(&mesh).unwrap(),
        PbrBundle {
            mesh: meshes.add(mesh),
            material: standard_materials.add(StandardMaterial {
                base_color: Color::GRAY,
                ..Default::default()
            }),
            transform: Transform::default().with_scale(Vec3::ONE * 5.0),
            ..Default::default()
        },
    ));
}
