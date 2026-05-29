use avian2d::math::{AdjustPrecision, Scalar, Vector};
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::animation::{AnimationState, AsepriteAnimation, Direction, load_aseprite_animation};
use crate::map::{BLOCK_POSITION, BLOCK_SIZE};

const IDLE_PATH: &str = "assets/stand_right_face_1hand.aseprite";
const WALK_PATH: &str = "assets/walk_right_face_1hand.aseprite";
const JUMP_PATH: &str = "assets/jump_right_face.aseprite";
const PLAYER_BODY_COLLIDER_SIZE: Vec2 = Vec2::new(19.0, 68.0);
const SPRITE_SCALE: f32 = 1.0;
const PLAYER_SPEED: f32 = 160.0;
const JUMP_SPEED: f32 = 500.0;
const JUMP_BUFFER_SECONDS: f32 = 0.05;
const GROUND_SENSOR_WIDTH: f32 = 20.0;
const GROUND_SENSOR_HEIGHT: f32 = 0.0;
const GROUND_SENSOR_DISTANCE: f32 = 4.0;
const GROUND_NORMAL_THRESHOLD: Scalar = 0.7;
const JUMP_ANIMATION_SPEED_THRESHOLD: Scalar = 20.0;

/// Marker for the player-controlled entity.
///
/// This struct has no fields on purpose. In Bevy, an empty component is often
/// used as a "tag" so queries can ask for only the entity that has `Player`.
#[derive(Component)]
pub struct Player;

/// Movement state that should persist between input frames.
///
/// The physics engine stores position and velocity, but gameplay input needs a
/// little extra state. `jump_buffer` remembers a jump request briefly so the
/// player can press jump just before landing and still jump on the first
/// grounded frame.
#[derive(Component, Default)]
pub struct PlayerPhysics {
    jump_buffer: f32,
}

/// Whether the player was touching a walkable surface during the latest physics update.
///
/// This is a tuple struct: `Grounded(true)` means grounded, `Grounded(false)`
/// means not grounded. Keeping it as a stable component avoids inserting and
/// removing a marker component every frame.
#[derive(Component, Default)]
pub struct Grounded(bool);

/// Creates the camera and the player entity.
///
/// `Commands` is Bevy's way to create, modify, or delete entities.
/// `ResMut<Assets<Image>>` gives mutable access to Bevy's image asset storage,
/// which is where the loaded Aseprite frames are registered.
pub fn setup_player(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);

    // All source art faces right. Facing left is handled later with
    // `sprite.flip_x`, so only one set of image files is needed.
    let animation = AsepriteAnimation::new(
        load_aseprite_animation(IDLE_PATH, &mut images),
        load_aseprite_animation(WALK_PATH, &mut images),
        load_aseprite_animation(JUMP_PATH, &mut images),
        Direction::Right,
        AnimationState::Idle,
    );
    let mut sprite = Sprite::from_image(animation.active_clip().frames[0].clone());
    sprite.flip_x = should_flip_sprite(animation.direction);

    // Start the player on top of the first block. The player transform uses the
    // entity center, so half the block height and half the player height are
    // added to the block's y position.
    let player_start = Vec3::new(
        BLOCK_POSITION.x,
        BLOCK_POSITION.y
            + (BLOCK_SIZE.y / 2.0)
            + (PLAYER_BODY_COLLIDER_SIZE.y * SPRITE_SCALE / 2.0),
        0.0,
    );

    commands.spawn((
        // Rendering.
        sprite,
        Transform::from_translation(player_start).with_scale(Vec3::splat(SPRITE_SCALE)),
        // Physics body and collision shape.
        RigidBody::Dynamic,
        Collider::rectangle(PLAYER_BODY_COLLIDER_SIZE.x, PLAYER_BODY_COLLIDER_SIZE.y),
        // A shape cast is a small sensor projected below the player. Avian fills
        // `ShapeHits` with whatever this sensor touches, which we use for
        // grounded checks instead of relying on exact collision positions.
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
        // Prevent the physics body from tipping over after collisions.
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity::ZERO,
        // Zero friction keeps horizontal movement controlled by our input code.
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        // Gameplay and animation components.
        Player,
        PlayerPhysics::default(),
        Grounded::default(),
        animation,
    ));
}

/// Applies keyboard input to the player's physics velocity.
///
/// `Query<(...), With<Player>>` means: find every entity that has `Player` and
/// give this system access to the listed components. `&mut` means the system can
/// modify that component; `&` means read-only access.
pub fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut LinearVelocity, &mut PlayerPhysics, &Grounded), With<Player>>,
) {
    let delta_secs = time.delta_secs_f64().adjust_precision();
    let horizontal_velocity = horizontal_direction(&keyboard) * PLAYER_SPEED;

    for (mut velocity, mut physics, grounded) in &mut query {
        // Count down the jump buffer every frame. `max(0.0)` prevents it from
        // becoming negative.
        physics.jump_buffer = (physics.jump_buffer - delta_secs).max(0.0);

        // `pressed` allows hold-to-repeat jumping. As long as space is held, the
        // buffer is refreshed; when the player touches ground, the jump fires.
        if keyboard.pressed(KeyCode::Space) {
            physics.jump_buffer = JUMP_BUFFER_SECONDS;
        }

        // Avian reads `LinearVelocity` and moves the physics body for us.
        velocity.x = horizontal_velocity;

        // A jump is allowed only while grounded. After applying jump velocity,
        // the buffer is cleared so one buffered press produces one jump.
        if grounded.0 && physics.jump_buffer > 0.0 {
            velocity.y = JUMP_SPEED;
            physics.jump_buffer = 0.0;
        }
    }
}

/// Updates the cached grounded flag from Avian's shape-cast results.
///
/// `ShapeHits` is produced by the `ShapeCaster` component added in
/// `setup_player`. We consider the player grounded if the sensor hit has an
/// upward-facing normal. A normal y value near `1.0` means the surface points
/// mostly upward, while values near `0.0` are more like walls.
pub fn update_grounded(mut query: Query<(&mut Grounded, &ShapeHits), With<Player>>) {
    for (mut grounded, hits) in &mut query {
        grounded.0 = hits
            .iter()
            .any(|hit| hit.normal1.y > GROUND_NORMAL_THRESHOLD);
    }
}

/// Chooses the current facing direction and animation state.
///
/// The name is a little historical: this now handles more than direction. It
/// also selects idle, walking, or jumping animation based on input and physics.
pub fn change_direction(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            &mut Sprite,
            &mut AsepriteAnimation,
            &Grounded,
            &LinearVelocity,
        ),
        With<Player>,
    >,
) {
    let horizontal_direction = horizontal_direction(&keyboard);
    let pressed_direction = pressed_direction(&keyboard);

    for (mut sprite, mut animation, grounded, velocity) in &mut query {
        // If no left/right key is pressed, keep facing the previous direction.
        let direction = pressed_direction.unwrap_or(animation.direction);
        let state = animation_state(
            grounded.0,
            horizontal_direction,
            velocity.y,
            animation.state,
        );

        if animation.direction == direction && animation.state == state {
            continue;
        }

        // This only resets frame timing when the animation state changes. A
        // direction-only change while jumping just flips the current frame.
        animation.set_direction_and_state(direction, state);
        sprite.image = animation.active_frame();
        sprite.flip_x = should_flip_sprite(direction);
    }
}

/// Returns whether the sprite should be mirrored horizontally.
///
/// The loaded sprite art already faces right. Left-facing movement reuses the
/// same frames with `Sprite::flip_x`.
fn should_flip_sprite(direction: Direction) -> bool {
    direction == Direction::Left
}

/// Maps physics and input into an animation state.
///
/// The jump state is intentionally "sticky": once jump animation has started,
/// it stays active until the player is grounded again. Without that rule, the
/// animation can restart around the top of the jump, where vertical velocity is
/// briefly close to zero.
fn animation_state(
    grounded: bool,
    horizontal_direction: Scalar,
    vertical_velocity: Scalar,
    current_state: AnimationState,
) -> AnimationState {
    if current_state == AnimationState::Jump && !grounded
        || vertical_velocity > JUMP_ANIMATION_SPEED_THRESHOLD
        || (!grounded && vertical_velocity.abs() > JUMP_ANIMATION_SPEED_THRESHOLD)
    {
        AnimationState::Jump
    } else if horizontal_direction != 0.0 {
        AnimationState::Walk
    } else {
        AnimationState::Idle
    }
}

/// Converts left/right keyboard input into a numeric direction.
///
/// The result is `-1.0` for left, `1.0` for right, and `0.0` when neither or
/// both directions are pressed.
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

/// Returns the last requested facing direction from keyboard input.
///
/// This is separate from `horizontal_direction` because animation direction is a
/// discrete enum, while movement velocity is a number.
fn pressed_direction(keyboard: &ButtonInput<KeyCode>) -> Option<Direction> {
    if keyboard.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        Some(Direction::Left)
    } else if keyboard.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        Some(Direction::Right)
    } else {
        None
    }
}
