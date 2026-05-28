use bevy::prelude::*;

use crate::animation::{AsepriteAnimation, Direction, load_aseprite_animation};
use crate::collision::{Collider, Solid, collides_at};
use crate::map::{BLOCK_POSITION, BLOCK_SIZE};

// const IDLE_BACK_PATH: &str = "assets/Swordsman_lvl1_Idle_back.aseprite";
const IDLE_BACK_PATH: &str = "assets/idle_001.aseprite";
//const IDLE_FRONT_PATH: &str = "assets/Swordsman_lvl1_Idle_front.aseprite";
const IDLE_FRONT_PATH: &str = "assets/idle_001.aseprite";
const IDLE_SIDE_PATH: &str = "assets/idle_001.aseprite";
const PLAYER_COLLIDER_SIZE: Vec2 = Vec2::new(45.0, 68.0);
const SPRITE_SCALE: f32 = 1.0;
const PLAYER_SPEED: f32 = 160.0;
const GRAVITY: f32 = 1000.0;
const JUMP_SPEED: f32 = 500.0;
const GROUND_CHECK_DISTANCE: f32 = 1.0;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerPhysics {
    velocity: Vec2,
    grounded: bool,
}

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
        BLOCK_POSITION.y + (BLOCK_SIZE.y / 2.0) + (PLAYER_COLLIDER_SIZE.y * SPRITE_SCALE / 2.0),
        0.0,
    );

    commands.spawn((
        sprite,
        Transform::from_translation(player_start).with_scale(Vec3::splat(SPRITE_SCALE)),
        Collider {
            size: PLAYER_COLLIDER_SIZE,
        },
        Player,
        PlayerPhysics {
            grounded: true,
            ..default()
        },
        animation,
    ));
}

pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    solids: Query<(&Transform, &Collider), (With<Solid>, Without<Player>)>,
    mut query: Query<(&mut Transform, &Collider, &mut PlayerPhysics), (With<Player>, Without<Solid>)>,
) {
    let delta_secs = time.delta_secs();
    let horizontal_movement = horizontal_direction(&keyboard) * PLAYER_SPEED * delta_secs;

    for (mut transform, collider, mut physics) in &mut query {
        if keyboard.just_pressed(KeyCode::Space) && physics.grounded {
            physics.velocity.y = JUMP_SPEED;
            physics.grounded = false;
        }

        physics.velocity.y -= GRAVITY * delta_secs;

        let next_x = transform.translation + Vec3::new(horizontal_movement, 0.0, 0.0);
        if !collides_at(next_x, transform.scale, collider, &solids) {
            transform.translation = next_x;
        }

        let next_y = transform.translation + Vec3::new(0.0, physics.velocity.y * delta_secs, 0.0);
        if !collides_at(next_y, transform.scale, collider, &solids) {
            transform.translation = next_y;
            physics.grounded = is_on_ground(&transform, collider, &solids);
        } else {
            if physics.velocity.y < 0.0 {
                physics.grounded = true;
            }
            physics.velocity.y = 0.0;
        }

        if physics.velocity.y <= 0.0 {
            physics.grounded = is_on_ground(&transform, collider, &solids);
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

fn horizontal_direction(keyboard: &ButtonInput<KeyCode>) -> f32 {
    let mut direction = 0.0;

    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction -= 1.0;
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction += 1.0;
    }

    direction
}

fn is_on_ground<F: bevy::ecs::query::QueryFilter>(
    transform: &Transform,
    collider: &Collider,
    solids: &Query<(&Transform, &Collider), F>,
) -> bool {
    let ground_probe = transform.translation + Vec3::new(0.0, -GROUND_CHECK_DISTANCE, 0.0);
    collides_at(ground_probe, transform.scale, collider, solids)
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
