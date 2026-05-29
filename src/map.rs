use avian2d::prelude::*;
use bevy::prelude::*;

pub const BLOCK_SIZE: Vec2 = Vec2::new(64.0, 64.0);
pub const BLOCK_POSITION: Vec3 = Vec3::new(120.0, 0.0, 0.0);
const BLOCK_POSITION_2: Vec3 = Vec3::new(-50.0, 0.0, 0.0);
const FLOOR_POSITION: Vec3 = Vec3::new(-50.0, -50.0, 0.0);
const FLOOR_SIZE: Vec2 = Vec2::new(1000.0, 20.0);
const BLOCK_COLOR: Color = Color::srgb(0.35, 0.28, 0.22);

#[derive(Clone, Copy)]
struct BlockSpec {
    position: Vec3,
    size: Vec2,
}

const BLOCKS: [BlockSpec; 3] = [
    BlockSpec {
        position: BLOCK_POSITION,
        size: BLOCK_SIZE,
    },
    BlockSpec {
        position: BLOCK_POSITION_2,
        size: BLOCK_SIZE,
    },
    BlockSpec {
        position: FLOOR_POSITION,
        size: FLOOR_SIZE,
    },
];

pub fn setup_map(mut commands: Commands) {
    for block in BLOCKS {
        commands.spawn((
            Sprite::from_color(BLOCK_COLOR, block.size),
            Transform::from_translation(block.position),
            RigidBody::Static,
            Collider::rectangle(block.size.x, block.size.y),
        ));
    }
}
