use avian2d::math::{AdjustPrecision, Scalar, Vector};
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::animation::{AsepriteAnimation, Direction, load_aseprite_animation};
use crate::map::{BLOCK_POSITION, BLOCK_SIZE};

// const IDLE_BACK_PATH: &str = "assets/Swordsman_lvl1_Idle_back.aseprite";
const IDLE_BACK_PATH: &str = "assets/idle_001.aseprite";
//const IDLE_FRONT_PATH: &str = "assets/Swordsman_lvl1_Idle_front.aseprite";
const IDLE_FRONT_PATH: &str = "assets/idle_001.aseprite";
const IDLE_SIDE_PATH: &str = "assets/idle_001.aseprite";
const PLAYER_BODY_COLLIDER_SIZE: Vec2 = Vec2::new(18.0, 68.0);
const SPRITE_SCALE: f32 = 1.0;
const PLAYER_SPEED: f32 = 160.0;
const JUMP_SPEED: f32 = 500.0;
const JUMP_BUFFER_SECONDS: f32 = 0.1;
const GROUND_SENSOR_WIDTH: f32 = 16.0;
const GROUND_SENSOR_HEIGHT: f32 = 4.0;
const GROUND_SENSOR_DISTANCE: f32 = 4.0;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerPhysics {
    jump_buffer: f32,
}

#[derive(Component)]
pub struct Grounded;

pub fn setup_player(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);

    let side_idle = load_aseprite_animation(IDLE_SIDE_PATH, &mut images);
    let animation = AsepriteAnimation::new(
        load_aseprite_animation(IDLE_BACK_PATH, &mut images),
        load_aseprite_animation(IDLE_FRONT_PATH, &mut images),
        side_idle.clone(),
        side_idle,
        Direction::Down,
    );
    let mut sprite = Sprite::from_image(animation.active_clip().frames[0].clone());
    sprite.flip_x = should_flip_sprite(animation.direction);
    let player_start = Vec3::new(
        BLOCK_POSITION.x,
        BLOCK_POSITION.y
            + (BLOCK_SIZE.y / 2.0)
            + (PLAYER_BODY_COLLIDER_SIZE.y * SPRITE_SCALE / 2.0),
        0.0,
    );

    commands.spawn((
        sprite,
        Transform::from_translation(player_start).with_scale(Vec3::splat(SPRITE_SCALE)),
        RigidBody::Dynamic,
        Collider::rectangle(PLAYER_BODY_COLLIDER_SIZE.x, PLAYER_BODY_COLLIDER_SIZE.y),
        ShapeCaster::new(
            Collider::rectangle(GROUND_SENSOR_WIDTH, GROUND_SENSOR_HEIGHT),
            Vector::new(
                0.0,
                -(PLAYER_BODY_COLLIDER_SIZE.y / 2.0) - (GROUND_SENSOR_HEIGHT / 2.0),
            ),
            0.0,
            Dir2::NEG_Y,
        )
        .with_max_distance(GROUND_SENSOR_DISTANCE),
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity::ZERO,
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        Player,
        PlayerPhysics::default(),
        animation,
    ));
}

pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut LinearVelocity, &mut PlayerPhysics, Option<&Grounded>), With<Player>>,
) {
    let delta_secs = time.delta_secs_f64().adjust_precision();
    let horizontal_velocity = horizontal_direction(&keyboard) * PLAYER_SPEED;

    for (mut velocity, mut physics, grounded) in &mut query {
        physics.jump_buffer = (physics.jump_buffer - delta_secs).max(0.0);
        if keyboard.just_pressed(KeyCode::Space) {
            physics.jump_buffer = JUMP_BUFFER_SECONDS;
        }

        velocity.x = horizontal_velocity;

        if grounded.is_some() && physics.jump_buffer > 0.0 {
            velocity.y = JUMP_SPEED;
            physics.jump_buffer = 0.0;
        }
    }
}

pub fn update_grounded(mut commands: Commands, query: Query<(Entity, &ShapeHits), With<Player>>) {
    for (entity, hits) in &query {
        let grounded = hits.iter().any(|hit| hit.normal1.y > 0.7);

        if grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

pub fn change_direction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Sprite, &mut AsepriteAnimation)>,
) {
    let Some(direction) = pressed_direction(&keyboard) else {
        return;
    };

    for (mut sprite, mut animation) in &mut query {
        if animation.direction == direction {
            continue;
        }

        animation.reset(direction);
        sprite.image = animation.active_clip().frames[0].clone();
        sprite.flip_x = should_flip_sprite(direction);
    }
}

fn should_flip_sprite(direction: Direction) -> bool {
    direction == Direction::Right
}

fn horizontal_direction(keyboard: &ButtonInput<KeyCode>) -> Scalar {
    let mut direction = 0.0;

    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction -= 1.0;
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction += 1.0;
    }

    direction
}

fn pressed_direction(keyboard: &ButtonInput<KeyCode>) -> Option<Direction> {
    if keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        Some(Direction::Up)
    } else if keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        Some(Direction::Down)
    } else if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        Some(Direction::Left)
    } else if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        Some(Direction::Right)
    } else {
        None
    }
}
