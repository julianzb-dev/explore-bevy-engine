use aseprite_loader::loader::{AsepriteFile, LayerSelection};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Clone)]
pub struct AnimationClip {
    /// Image handles for each rendered frame.
    ///
    /// `Handle<Image>` is Bevy's lightweight reference to an image stored in
    /// `Assets<Image>`. Cloning a handle does not clone the image pixels.
    pub frames: Vec<Handle<Image>>,
    /// How long each frame should stay on screen, in seconds.
    frame_durations: Vec<f32>,
}

/// Cardinal facing direction used to flip right-facing source art.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

/// High-level animation mode for the player.
///
/// Direction is separate from state because the same walking animation can face
/// left or right by flipping the sprite.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    Walk,
    Jump,
}

/// Component that stores the animation clips and playback cursor for one entity.
///
/// In Bevy, systems advance animation by mutating this component and the
/// entity's `Sprite`. The component owns the timing state: current frame and
/// elapsed time within that frame.
#[derive(Component)]
pub struct AsepriteAnimation {
    idle: AnimationClip,
    walk: AnimationClip,
    jump: AnimationClip,
    pub direction: Direction,
    pub state: AnimationState,
    current_frame: usize,
    elapsed: f32,
}

impl AsepriteAnimation {
    /// Builds an animation component from the three clips used by the player.
    pub fn new(
        idle: AnimationClip,
        walk: AnimationClip,
        jump: AnimationClip,
        direction: Direction,
        state: AnimationState,
    ) -> Self {
        Self {
            idle,
            walk,
            jump,
            direction,
            state,
            current_frame: 0,
            elapsed: 0.0,
        }
    }

    /// Updates facing direction and animation state.
    ///
    /// The animation only restarts when the state changes. For example, changing
    /// from `Idle` to `Walk` resets to frame 0, but changing from facing right
    /// to facing left during `Jump` keeps the current jump frame.
    pub fn set_direction_and_state(&mut self, direction: Direction, state: AnimationState) {
        let state_changed = self.state != state;

        self.direction = direction;
        self.state = state;

        if state_changed {
            self.current_frame = 0;
            self.elapsed = 0.0;
        }
    }

    /// Returns the clip that matches the current animation state.
    pub fn active_clip(&self) -> &AnimationClip {
        match self.state {
            AnimationState::Idle => &self.idle,
            AnimationState::Walk => &self.walk,
            AnimationState::Jump => &self.jump,
        }
    }

    /// Returns a handle to the image frame currently displayed by this animation.
    pub fn active_frame(&self) -> Handle<Image> {
        self.active_clip().frames[self.current_frame].clone()
    }

    /// Idle and walk loop forever; jump plays once and freezes on its last frame.
    fn should_loop(&self) -> bool {
        self.state != AnimationState::Jump
    }
}

/// Loads an Aseprite file into Bevy image assets.
///
/// The `aseprite-loader` crate parses and renders each Aseprite frame into raw
/// RGBA pixels. Those pixels are then inserted into `Assets<Image>`, and Bevy
/// gives us a handle for each frame.
pub fn load_aseprite_animation(path: &str, images: &mut Assets<Image>) -> AnimationClip {
    let data = std::fs::read(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    let aseprite = AsepriteFile::load(&data)
        .unwrap_or_else(|error| panic!("failed to parse Aseprite file {path}: {error}"));
    let (width, height) = aseprite.size();
    let frame_size = width as usize * height as usize * 4;

    let frames = (0..aseprite.frames().len())
        .map(|frame_index| {
            // Allocate a pixel buffer for one rendered frame. Each pixel needs
            // four bytes: red, green, blue, and alpha.
            let mut rgba = vec![0u8; frame_size];
            aseprite
                .render_frame(frame_index, &mut rgba, &LayerSelection::Visible)
                .unwrap_or_else(|error| {
                    panic!("failed to render frame {frame_index} from {path}: {error}")
                });

            images.add(Image::new(
                Extent3d {
                    width: width.into(),
                    height: height.into(),
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                rgba,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            ))
        })
        .collect();

    // Aseprite stores frame durations in milliseconds. The animation system
    // works in seconds because Bevy's `Time::delta_secs()` returns seconds.
    let frame_durations = aseprite
        .frames()
        .iter()
        .map(|frame| frame.duration as f32 / 1000.0)
        .collect();

    AnimationClip {
        frames,
        frame_durations,
    }
}

/// Advances all `AsepriteAnimation` components by the elapsed frame time.
///
/// This system changes the entity's `Sprite::image` handle when enough time has
/// passed to move to the next animation frame.
pub fn execute_animations(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut AsepriteAnimation)>,
) {
    for (mut sprite, mut animation) in &mut query {
        let frame_count = animation.active_clip().frames.len();
        if frame_count <= 1 {
            continue;
        }

        animation.elapsed += time.delta_secs();

        while animation.elapsed >= animation.active_clip().frame_durations[animation.current_frame]
        {
            animation.elapsed -= animation.active_clip().frame_durations[animation.current_frame];

            // Idle/walk wrap back to frame 0. Jump clamps to the last frame so
            // it does not restart while the player is still airborne.
            let next_frame = animation.current_frame + 1;
            if next_frame >= frame_count {
                if animation.should_loop() {
                    animation.current_frame = 0;
                } else {
                    animation.current_frame = frame_count - 1;
                    animation.elapsed = 0.0;
                }
            } else {
                animation.current_frame = next_frame;
            }

            sprite.image = animation.active_clip().frames[animation.current_frame].clone();

            if !animation.should_loop() && animation.current_frame == frame_count - 1 {
                break;
            }
        }
    }
}
