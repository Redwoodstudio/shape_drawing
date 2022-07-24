use bevy::prelude::Vec2;
use bevy_prototype_lyon::prelude::tess::math::Point;

pub fn point_from_positions(mouse: Vec2, origin: Vec2) -> Point {
    Point::new(mouse.x - origin.x, mouse.y - origin.y)
}

pub fn rotate_around_pivot(mouse: Vec2, origin: Vec2, pivot: Vec2) -> Point {
    Point::new(
        -mouse.x - origin.x + 2.0 * pivot.x,
        -mouse.y - origin.y + 2.0 * pivot.y,
    )
}
