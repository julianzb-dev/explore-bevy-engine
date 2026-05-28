mod animation;
mod map;
mod player;
mod window;

use animation::execute_animations;
use avian2d::prelude::*;
use bevy::prelude::*;
use map::setup_map;
use player::{change_direction, move_player, setup_player, update_grounded};
use window::window_plugin;

const GRAVITY: f32 = 1500.0;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(window_plugin())
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(PhysicsPlugins::default().with_length_unit(20.0))
        .insert_resource(Gravity(Vec2::NEG_Y * GRAVITY))
        .add_systems(Startup, (setup_player, setup_map))
        .add_systems(
            Update,
            (
                change_direction,
                update_grounded,
                move_player,
                execute_animations,
            )
                .chain(),
        )
        .run();
}
