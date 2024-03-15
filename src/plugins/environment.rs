use bevy::{
    app::App,
    asset::Assets,
    ecs::{
        schedule::OnEnter,
        system::{Commands, ResMut},
    },
    math::{primitives::Plane3d, Vec3},
    pbr::{
        CascadeShadowConfigBuilder, DirectionalLight, DirectionalLightBundle, PbrBundle,
        StandardMaterial,
    },
    render::{
        color::Color,
        mesh::{Mesh, Meshable},
    },
    transform::components::Transform,
};
use bevy_xpbd_3d::{components::RigidBody, plugins::collision::Collider};

use crate::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Game), setup);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn floor
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
            // transform: Transform::from_xyz(0.0, -15.0, 0.0),
            ..Default::default()
        },
    ));

    // Spawn light
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..Default::default()
    }
    .build();

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        cascade_shadow_config,
        ..Default::default()
    });
}
