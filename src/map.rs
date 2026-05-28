use avian2d::prelude::*;
use bevy::prelude::*;

pub const BLOCK_SIZE: Vec2 = Vec2::new(64.0, 64.0);
pub const BLOCK_POSITION: Vec3 = Vec3::new(120.0, 0.0, 0.0);
pub const BLOCK_POSITION_2: Vec3 = Vec3::new(-50.0, 0.0, 0.0);
pub const BLOCK_POSITION_3: Vec3 = Vec3::new(-50.0, -50.0, 0.0);

pub fn setup_map(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.35, 0.28, 0.22), BLOCK_SIZE),
        Transform::from_translation(BLOCK_POSITION),
        RigidBody::Static,
        Collider::rectangle(BLOCK_SIZE.x, BLOCK_SIZE.y),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.35, 0.28, 0.22), BLOCK_SIZE),
        Transform::from_translation(BLOCK_POSITION_2),
        RigidBody::Static,
        Collider::rectangle(BLOCK_SIZE.x, BLOCK_SIZE.y),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.35, 0.28, 0.22), Vec2::new(1000.0, 20.0)),
        Transform::from_translation(BLOCK_POSITION_3),
        RigidBody::Static,
        Collider::rectangle(1000.0, 20.0),
    ));
}
