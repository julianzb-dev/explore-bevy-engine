use bevy::prelude::*;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Basic 2D Map".into(),
            resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
            resizable: false,
            ..default()
        }),
        ..default()
    }
}
