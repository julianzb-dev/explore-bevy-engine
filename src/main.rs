mod animation;
mod player;
mod window;

use animation::execute_animations;
use bevy::prelude::*;
use player::{change_direction, setup_player};
use window::window_plugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(window_plugin())
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup_player)
        .add_systems(Update, (change_direction, execute_animations).chain())
        .run();
}
