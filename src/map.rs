use bevy::prelude::*;

use crate::collision::{Collider, Solid};

const BLOCK_SIZE: Vec2 = Vec2::new(64.0, 64.0);
const BLOCK_POSITION: Vec3 = Vec3::new(120.0, 0.0, 0.0);

pub fn setup_map(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.35, 0.28, 0.22), BLOCK_SIZE),
        Transform::from_translation(BLOCK_POSITION),
        Collider { size: BLOCK_SIZE },
        Solid,
    ));
}
