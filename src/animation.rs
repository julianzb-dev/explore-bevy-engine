use aseprite_loader::loader::{AsepriteFile, LayerSelection};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Clone)]
pub struct AnimationClip {
    pub frames: Vec<Handle<Image>>,
    frame_durations: Vec<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct AsepriteAnimation {
    pub up: AnimationClip,
    pub down: AnimationClip,
    pub left: AnimationClip,
    pub right: AnimationClip,
    pub direction: Direction,
    current_frame: usize,
    elapsed: f32,
}

impl AsepriteAnimation {
    pub fn new(
        up: AnimationClip,
        down: AnimationClip,
        left: AnimationClip,
        right: AnimationClip,
        direction: Direction,
    ) -> Self {
        Self {
            up,
            down,
            left,
            right,
            direction,
            current_frame: 0,
            elapsed: 0.0,
        }
    }

    pub fn reset(&mut self, direction: Direction) {
        self.direction = direction;
        self.current_frame = 0;
        self.elapsed = 0.0;
    }

    pub fn active_clip(&self) -> &AnimationClip {
        match self.direction {
            Direction::Up => &self.up,
            Direction::Down => &self.down,
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        }
    }
}

pub fn load_aseprite_animation(path: &str, images: &mut Assets<Image>) -> AnimationClip {
    let data = std::fs::read(path).expect("failed to read Aseprite file");
    let aseprite = AsepriteFile::load(&data).expect("failed to parse Aseprite file");
    let (width, height) = aseprite.size();
    let frame_size = width as usize * height as usize * 4;

    let frames = (0..aseprite.frames().len())
        .map(|frame_index| {
            let mut rgba = vec![0u8; frame_size];
            aseprite
                .render_frame(frame_index, &mut rgba, &LayerSelection::Visible)
                .expect("failed to render Aseprite frame");

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

pub fn execute_animations(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut AsepriteAnimation)>,
) {
    for (mut sprite, mut animation) in &mut query {
        if animation.active_clip().frames.len() <= 1 {
            continue;
        }

        animation.elapsed += time.delta_secs();

        while animation.elapsed >= animation.active_clip().frame_durations[animation.current_frame]
        {
            animation.elapsed -= animation.active_clip().frame_durations[animation.current_frame];
            animation.current_frame =
                (animation.current_frame + 1) % animation.active_clip().frames.len();
            sprite.image = animation.active_clip().frames[animation.current_frame].clone();
        }
    }
}
