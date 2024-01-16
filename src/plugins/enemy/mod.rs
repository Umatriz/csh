use bevy::{
    core::Name,
    ecs::{bundle::Bundle, component::Component},
    reflect::{std_traits::ReflectDefault, Reflect},
};

#[derive(Debug, Component, Reflect)]
#[reflect(Default)]
pub struct Health(u64);

impl Default for Health {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Default)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub name: Name,
    pub health: Health,
    pub enemy: Enemy,
}
