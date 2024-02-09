use bevy::ecs::system::Resource;
use clap::Parser;

#[derive(Parser, Resource, Clone, Debug)]
pub struct Args {
    /// runs the game in synctest mode
    #[clap(long)]
    pub synctest: bool,
}
