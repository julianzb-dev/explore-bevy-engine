use bevy::prelude::*;

use crate::animation::{AsepriteAnimation, Direction, load_aseprite_animation};
use crate::collision::{Collider, Solid, collides_at, scaled_collider_size};
use crate::map::{BLOCK_POSITION, BLOCK_SIZE};

// const IDLE_BACK_PATH: &str = "assets/Swordsman_lvl1_Idle_back.aseprite";
const IDLE_BACK_PATH: &str = "assets/idle_001.aseprite";
//const IDLE_FRONT_PATH: &str = "assets/Swordsman_lvl1_Idle_front.aseprite";
const IDLE_FRONT_PATH: &str = "assets/idle_001.aseprite";
const IDLE_SIDE_PATH: &str = "assets/idle_001.aseprite";
const PLAYER_COLLIDER_SIZE: Vec2 = Vec2::new(45.0, 68.0);
const SPRITE_SCALE: f32 = 1.0;
const PLAYER_SPEED: f32 = 160.0;
const GRAVITY: f32 = 1500.0;
const JUMP_SPEED: f32 = 500.0;
const JUMP_BUFFER_SECONDS: f32 = 0.1;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct PlayerPhysics {
    velocity: Vec2,
    grounded: bool,
    jump_buffer: f32,
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
    mut query: Query<
        (&mut Transform, &Collider, &mut PlayerPhysics),
        (With<Player>, Without<Solid>),
    >,
) {
    let delta_secs = time.delta_secs();
    let horizontal_movement = horizontal_direction(&keyboard) * PLAYER_SPEED * delta_secs;

    for (mut transform, collider, mut physics) in &mut query {
        physics.jump_buffer = (physics.jump_buffer - delta_secs).max(0.0);
        if keyboard.just_pressed(KeyCode::Space) {
            physics.jump_buffer = JUMP_BUFFER_SECONDS;
        }

        if physics.grounded && physics.jump_buffer > 0.0 {
            physics.velocity.y = JUMP_SPEED;
            physics.grounded = false;
            physics.jump_buffer = 0.0;
        }

        physics.grounded = false;
        physics.velocity.y -= GRAVITY * delta_secs;

        let next_x = transform.translation + Vec3::new(horizontal_movement, 0.0, 0.0);
        if !collides_at(next_x, transform.scale, collider, &solids) {
            transform.translation = next_x;
        }

        let vertical_movement = physics.velocity.y * delta_secs;
        let vertical_collision =
            resolve_vertical_movement(&mut transform, collider, vertical_movement, &solids);

        if let Some(collision) = vertical_collision {
            if collision == VerticalCollision::Ground {
                physics.grounded = true;
            }
            physics.velocity.y = 0.0;

            if physics.grounded && physics.jump_buffer > 0.0 {
                physics.velocity.y = JUMP_SPEED;
                physics.grounded = false;
                physics.jump_buffer = 0.0;
            }
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum VerticalCollision {
    Ground,
    Ceiling,
}

fn resolve_vertical_movement<F: bevy::ecs::query::QueryFilter>(
    transform: &mut Transform,
    collider: &Collider,
    movement_y: f32,
    solids: &Query<(&Transform, &Collider), F>,
) -> Option<VerticalCollision> {
    if movement_y == 0.0 {
        return None;
    }

    let player_size = scaled_collider_size(collider, transform.scale);
    let player_half = player_size / 2.0;
    let current_position = transform.translation;
    let next_y = current_position.y + movement_y;
    let current_bottom = current_position.y - player_half.y;
    let current_top = current_position.y + player_half.y;
    let next_bottom = next_y - player_half.y;
    let next_top = next_y + player_half.y;

    if movement_y < 0.0 {
        let mut highest_ground = None;

        for (solid_transform, solid_collider) in solids {
            let solid_size = scaled_collider_size(solid_collider, solid_transform.scale);
            let solid_half = solid_size / 2.0;
            let solid_left = solid_transform.translation.x - solid_half.x;
            let solid_right = solid_transform.translation.x + solid_half.x;
            let solid_top = solid_transform.translation.y + solid_half.y;
            let overlaps_x = current_position.x - player_half.x < solid_right
                && current_position.x + player_half.x > solid_left;

            if overlaps_x && current_bottom >= solid_top && next_bottom <= solid_top {
                highest_ground =
                    Some(highest_ground.map_or(solid_top, |ground: f32| ground.max(solid_top)));
            }
        }

        if let Some(ground_y) = highest_ground {
            transform.translation.y = ground_y + player_half.y;
            return Some(VerticalCollision::Ground);
        }
    } else {
        let mut lowest_ceiling = None;

        for (solid_transform, solid_collider) in solids {
            let solid_size = scaled_collider_size(solid_collider, solid_transform.scale);
            let solid_half = solid_size / 2.0;
            let solid_left = solid_transform.translation.x - solid_half.x;
            let solid_right = solid_transform.translation.x + solid_half.x;
            let solid_bottom = solid_transform.translation.y - solid_half.y;
            let overlaps_x = current_position.x - player_half.x < solid_right
                && current_position.x + player_half.x > solid_left;

            if overlaps_x && current_top <= solid_bottom && next_top >= solid_bottom {
                lowest_ceiling = Some(
                    lowest_ceiling.map_or(solid_bottom, |ceiling: f32| ceiling.min(solid_bottom)),
                );
            }
        }

        if let Some(ceiling_y) = lowest_ceiling {
            transform.translation.y = ceiling_y - player_half.y;
            return Some(VerticalCollision::Ceiling);
        }
    }

    transform.translation.y = next_y;
    None
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
