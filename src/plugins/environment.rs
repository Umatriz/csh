use bevy::{
    app::App,
    asset::Assets,
    ecs::{
        schedule::OnEnter,
        system::{Commands, ResMut},
    },
    math::{primitives::Plane3d, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
};

use crate::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), spawn_floor);
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::new(Vec3::Y)),
        material: standard_materials.add(StandardMaterial {
            base_color: Color::GRAY,
            ..Default::default()
        }),
        transform: Transform::default().with_scale(Vec3::ONE * 5.0),
        ..Default::default()
    });
}
