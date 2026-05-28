mod animation;
mod collision;
mod map;
mod player;
mod window;

use animation::execute_animations;
use bevy::prelude::*;
use map::setup_map;
use player::{change_direction, move_player, setup_player};
use window::window_plugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(window_plugin())
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, (setup_player, setup_map))
        .add_systems(Update, (change_direction, move_player, execute_animations).chain())
        .run();
}
