use bevy::prelude::*;
use bevy_spritefusion::prelude::*;

// https://github.com/Hugo-Dz/bevy_spritefusion

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(SpriteFusionPlugin)
        .add_systems(Startup, spawn_map)
        .add_systems(Update, print_collectibles)
        .run();
}

fn spawn_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(300.0, 50.0, 0.0),
        Projection::Orthographic(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));
    commands.spawn(SpriteFusionBundle {
        map: SpriteFusionMapHandle(asset_server.load("map.json")),  
        tileset: SpriteFusionTilesetHandle(asset_server.load("spritesheet.png")),
        ..default()
    });
}

/// Access the tile custom attributes you can set in Sprite Fusion.
fn print_collectibles(query: Query<(&TilePos, &TileAttributes)>, mut has_run: Local<bool>) {
    if query.is_empty() || *has_run {
        return;
    }
    *has_run = true;

    info!("Tiles with attributes:");
    for (pos, attrs) in query.iter() {
        if let Some(name) = attrs.get_str("name") {
            let value = attrs.get_i64("value").unwrap_or(0);
            let is_collectible = attrs.get_bool("isCollectible").unwrap_or(false);
            info!(
                "  - '{}' at ({}, {}), value: {}, collectible: {}",
                name, pos.x, pos.y, value, is_collectible
            );
        }
    }
}