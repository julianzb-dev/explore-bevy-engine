use bevy::prelude::*;
use bevy::ecs::query::QueryFilter;

#[derive(Component)]
pub struct Collider {
    pub size: Vec2,
}

#[derive(Component)]
pub struct Solid;

pub fn collides_at<F: QueryFilter>(
    position: Vec3,
    collider: &Collider,
    solids: &Query<(&Transform, &Collider), F>,
) -> bool {
    solids.iter().any(|(solid_transform, solid_collider)| {
        aabb_intersects(
            position.truncate(),
            collider.size,
            solid_transform.translation.truncate(),
            solid_collider.size,
        )
    })
}

fn aabb_intersects(a_position: Vec2, a_size: Vec2, b_position: Vec2, b_size: Vec2) -> bool {
    let a_half = a_size / 2.0;
    let b_half = b_size / 2.0;

    a_position.x - a_half.x < b_position.x + b_half.x
        && a_position.x + a_half.x > b_position.x - b_half.x
        && a_position.y - a_half.y < b_position.y + b_half.y
        && a_position.y + a_half.y > b_position.y - b_half.y
}
