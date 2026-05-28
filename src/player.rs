use bevy::prelude::*;

use crate::animation::{AsepriteAnimation, Direction, load_aseprite_animation};

const IDLE_BACK_PATH: &str = "assets/Swordsman_lvl1_Idle_back.aseprite";
const IDLE_FRONT_PATH: &str = "assets/Swordsman_lvl1_Idle_front.aseprite";
const IDLE_SIDE_PATH: &str = "assets/idle_001.aseprite";
const SPRITE_SCALE: f32 = 2.0;

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
        animation,
    ));
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
