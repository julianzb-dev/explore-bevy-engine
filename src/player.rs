use bevy::prelude::*;

use crate::animation::{AsepriteAnimation, Direction, load_aseprite_animation};
use crate::collision::{Collider, Solid, collides_at};

// const IDLE_BACK_PATH: &str = "assets/Swordsman_lvl1_Idle_back.aseprite";
const IDLE_BACK_PATH: &str = "assets/idle_001.aseprite";
//const IDLE_FRONT_PATH: &str = "assets/Swordsman_lvl1_Idle_front.aseprite";
const IDLE_FRONT_PATH: &str = "assets/idle_001.aseprite";
const IDLE_SIDE_PATH: &str = "assets/idle_001.aseprite";
const PLAYER_COLLIDER_SIZE: Vec2 = Vec2::new(45.0, 68.0);
const SPRITE_SCALE: f32 = 1.0;
const PLAYER_SPEED: f32 = 160.0;

#[derive(Component)]
pub struct Player;

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

    commands.spawn((
        sprite,
        Transform::from_scale(Vec3::splat(SPRITE_SCALE)),
        Collider {
            size: PLAYER_COLLIDER_SIZE,
        },
        Player,
        animation,
    ));
}

pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    solids: Query<(&Transform, &Collider), (With<Solid>, Without<Player>)>,
    mut query: Query<(&mut Transform, &Collider), (With<Player>, Without<Solid>)>,
) {
    let Some(direction) = movement_direction(&keyboard) else {
        return;
    };

    let movement = direction * PLAYER_SPEED * time.delta_secs();

    for (mut transform, collider) in &mut query {
        let next_x = transform.translation + Vec3::new(movement.x, 0.0, 0.0);
        if !collides_at(next_x, collider, &solids) {
            transform.translation = next_x;
        }

        let next_y = transform.translation + Vec3::new(0.0, movement.y, 0.0);
        if !collides_at(next_y, collider, &solids) {
            transform.translation = next_y;
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

fn movement_direction(keyboard: &ButtonInput<KeyCode>) -> Option<Vec3> {
    let mut direction = Vec3::ZERO;

    if keyboard.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        direction.y += 1.0;
    }
    if keyboard.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        direction.y -= 1.0;
    }
    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        direction.x -= 1.0;
    }
    if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        direction.x += 1.0;
    }

    (direction != Vec3::ZERO).then(|| direction.normalize())
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
